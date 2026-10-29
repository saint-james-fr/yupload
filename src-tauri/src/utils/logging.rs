/// Debug logging macro that only logs in debug builds
///
/// This macro wraps log::debug! and only executes in debug builds.
/// In release builds, the code is completely removed for zero performance impact.
///
/// Usage:
/// ```rust
/// tracing::debug!("Response body: {}", response_text);
/// ```
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            log::debug!($($arg)*);
        }
    };
}

/// Error logging macro
///
/// Always logs errors, regardless of build type.
///
/// Usage:
/// ```rust
/// tracing::error!("Failed to process: {}", error);
/// ```
#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}

/// Info logging macro
///
/// Usage:
/// ```rust
/// tracing::info!("Processing request");
/// ```
#[macro_export]
macro_rules! info_log {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

/// Warn logging macro
///
/// Usage:
/// ```rust
/// tracing::warn!("Rate limit approaching");
/// ```
#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}
