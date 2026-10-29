fn main() {
    // Load .env file at build time to inject environment variables into the binary
    // This allows bundled apps to have credentials baked in without needing .env at runtime
    if let Err(e) = dotenvy::dotenv() {
        // .env file is optional - warn but don't fail
        println!("cargo:warning=Could not load .env file: {}", e);
        println!("cargo:warning=Environment variables will be read from system or use defaults");
    }

    // Emit environment variables so they're available at compile time via env!() macro
    // These are baked into the binary during compilation
    if let Ok(client_id) = std::env::var("YOUTUBE_CLIENT_ID") {
        println!("cargo:rustc-env=YOUTUBE_CLIENT_ID_BUILD={}", client_id);
    } else {
        println!("cargo:warning=YOUTUBE_CLIENT_ID not found in .env, will use fallback");
    }

    if let Ok(key) = std::env::var("DISCOGS_CONSUMER_KEY") {
        println!("cargo:rustc-env=DISCOGS_CONSUMER_KEY_BUILD={}", key);
    }

    if let Ok(secret) = std::env::var("DISCOGS_CONSUMER_SECRET") {
        println!("cargo:rustc-env=DISCOGS_CONSUMER_SECRET_BUILD={}", secret);
    }

    tauri_build::build()
}
