#[macro_use]
pub mod utils;

pub mod commands;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod state;
pub mod use_cases;

use infrastructure::OAuthCallbackServer;
use state::{DiscogsClientState, YouTubeClientState};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

/// Setup cleanup task for temporary files
async fn setup_cleanup_task() {
    use utils::{cleanup::TEMP_FILE_MAX_AGE_HOURS, cleanup_old_temp_files};

    // Run cleanup immediately on startup
    let _ = cleanup_old_temp_files("yupload_", TEMP_FILE_MAX_AGE_HOURS);
    let _ = cleanup_old_temp_files("processed_image", TEMP_FILE_MAX_AGE_HOURS);
    let _ = cleanup_old_temp_files("thumb_", TEMP_FILE_MAX_AGE_HOURS);
    let _ = cleanup_old_temp_files("video_thumb_", TEMP_FILE_MAX_AGE_HOURS);
    let _ = cleanup_old_temp_files("normalized_", TEMP_FILE_MAX_AGE_HOURS);

    tracing::info!("Temp file cleanup initialized");

    // Setup periodic cleanup every hour
    loop {
        tokio::time::sleep(crate::utils::cleanup::TEMP_FILE_CLEANUP_INTERVAL).await;

        let total_cleaned = cleanup_old_temp_files("yupload_", TEMP_FILE_MAX_AGE_HOURS)
            .unwrap_or(0)
            + cleanup_old_temp_files("processed_image", TEMP_FILE_MAX_AGE_HOURS).unwrap_or(0)
            + cleanup_old_temp_files("thumb_", TEMP_FILE_MAX_AGE_HOURS).unwrap_or(0)
            + cleanup_old_temp_files("video_thumb_", TEMP_FILE_MAX_AGE_HOURS).unwrap_or(0)
            + cleanup_old_temp_files("normalized_", TEMP_FILE_MAX_AGE_HOURS).unwrap_or(0);

        if total_cleaned > 0 {
            tracing::info!("Periodic cleanup: removed {} old temp files", total_cleaned);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Note: Logging is now initialized in the setup function to access app data directory
    
    // Set up panic hook to log panics before the app crashes
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown".to_string());
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        
        // Use eprintln as a fallback since tracing might not be initialized yet
        eprintln!("PANIC: {} at {}", message, location);
        
        // Try to log with tracing if available
        tracing::error!("Application panic: {} at {}", message, location);
    }));

    // Load environment variables from .env file (if it exists)
    // This is optional - environment variables can also be set system-wide
    let _ = dotenvy::dotenv();

    let youtube_client_state: YouTubeClientState = Arc::new(tokio::sync::Mutex::new(None));
    let discogs_client_state: DiscogsClientState = Arc::new(tokio::sync::Mutex::new(None));
    let cancellation_state: state::CancellationState = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // Start OAuth callback server at app launch
    let oauth_server = match OAuthCallbackServer::start() {
        Ok(server) => {
            eprintln!("OAuth callback server started successfully");
            Arc::new(server)
        },
        Err(e) => {
            eprintln!("Failed to start OAuth callback server: {}", e);
            panic!("Failed to start OAuth callback server: {}", e);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_deep_link::init())
        .manage(youtube_client_state)
        .manage(discogs_client_state)
        .manage(oauth_server)
        .manage(cancellation_state)
        .setup(|app| {
            // Initialize logging system
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            if let Err(e) = infrastructure::logging::init_logging(app_data_dir) {
                eprintln!("Failed to initialize logging: {}", e);
                // Fall back to env_logger if tracing fails
                env_logger::Builder::from_default_env()
                    .format_timestamp(None)
                    .format_module_path(false)
                    .init();
            }

            tracing::info!("Yupload application starting up...");
            tracing::info!("OAuth callback server started successfully");

            use utils::window::{MAIN_HEIGHT, MAIN_WIDTH};
            // Create main window from Rust for macOS transparent titlebar + custom background
            // See: https://v2.tauri.app/learn/window-customization/#macos-transparent-titlebar-with-custom-window-background-color
            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("Yupload")
                .inner_size(MAIN_WIDTH, MAIN_HEIGHT)
                .min_inner_size(MAIN_WIDTH, MAIN_HEIGHT)
                .resizable(false);

            #[cfg(target_os = "macos")]
            let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

            let window = win_builder.build()?;

            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSColor, NSWindow};

                let ptr = window.ns_window()?;
                unsafe {
                    let ns_window: &NSWindow = &*ptr.cast();
                    let bg_color =
                        NSColor::colorWithDeviceRed_green_blue_alpha(20.0 / 255.0, 20.0 / 255.0, 20.0 / 255.0, 1.0);
                    ns_window.setBackgroundColor(Some(&bg_color));
                }
            }

            // On Linux/Windows, register deep-link schemes at runtime so they work in dev
            // and when the app is not installed (e.g. AppImage). macOS requires config-only.
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }
            // Register cleanup handler for temp files on app exit
            // Setup periodic cleanup task
            tauri::async_runtime::spawn(async move {
                setup_cleanup_task().await;
            });

            // Register window close event handler for exit cleanup
            window.on_window_event(move |event| {
                use tauri::WindowEvent;
                if let WindowEvent::CloseRequested { .. } = event {
                    tracing::info!("Window close requested - running exit cleanup");
                    tauri::async_runtime::spawn(async move {
                        // Run one final cleanup pass for all temp file prefixes
                        use utils::cleanup_old_temp_files;
                        // Use age 0 to clean up all temp files regardless of age
                        let _ = cleanup_old_temp_files("yupload_", 0);
                        let _ = cleanup_old_temp_files("processed_image", 0);
                        let _ = cleanup_old_temp_files("thumb_", 0);
                        let _ = cleanup_old_temp_files("video_thumb_", 0);
                        let _ = cleanup_old_temp_files("normalized_", 0);
                        tracing::info!("Exit cleanup completed");
                    });
                }
            });

            tracing::info!("Application setup completed successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // File commands
            commands::select_image,
            commands::select_audio_files,
            commands::select_folder,
            commands::select_output_folder,
            commands::list_folder_contents,
            commands::fetch_image_as_data_url,
            commands::get_temp_dir,
            commands::cleanup_temp_videos,
            // Audio commands
            commands::get_audio_title,
            commands::get_audio_metadata,
            commands::normalize_audio_files,
            commands::normalize_audio_files_standalone,
            // Video commands
            commands::create_video,
            commands::create_bulk_videos,
            commands::generate_image_preview,
            commands::generate_image_thumbnail,
            commands::list_predefined_backgrounds,
            commands::list_user_backgrounds,
            commands::save_predefined_background,
            commands::generate_video_thumbnail,
            // YouTube commands
            commands::authenticate_youtube,
            commands::check_youtube_auth,
            commands::list_youtube_playlists,
            commands::upload_video_to_youtube,
            commands::upload_bulk_videos_to_youtube,
            commands::disconnect_youtube,
            commands::check_video_processing_status,
            // Discogs commands
            commands::authenticate_discogs,
            commands::check_discogs_auth,
            commands::search_discogs_releases,
            commands::get_discogs_release,
            commands::get_discogs_master,
            commands::generate_description_from_release,
            commands::disconnect_discogs,
            // Store commands
            commands::get_app_preferences,
            commands::set_app_preferences,
            commands::save_upload_history,
            commands::get_upload_history,
            commands::clear_all_data,
            // Cancel commands
            commands::cancel_operation,
            commands::cancel_item,
            // System commands
            commands::get_log_files,
            commands::export_logs,
            commands::get_current_log_content,
            commands::get_system_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
