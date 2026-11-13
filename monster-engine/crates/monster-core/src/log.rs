//! Leveled logging to stderr, zero-dependency.
//!
//! Usage: `monster_core::log_info!("engine started");`
//! Level threshold can be set programmatically or via the
//! `MONSTER_LOG` environment variable (`error|warn|info|debug|trace`).

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// Severity levels, ordered from most to least severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    fn parse(s: &str) -> Option<Level> {
        match s.trim().to_ascii_lowercase().as_str() {
            "error" => Some(Level::Error),
            "warn" => Some(Level::Warn),
            "info" => Some(Level::Info),
            "debug" => Some(Level::Debug),
            "trace" => Some(Level::Trace),
            _ => None,
        }
    }
}

static MAX_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);
static INIT_DONE: OnceLock<()> = OnceLock::new();

/// Set the maximum level that will be printed.
pub fn set_max_level(level: Level) {
    MAX_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Whether a message at `level` passes the current threshold.
pub fn should_log(level: Level) -> bool {
    (level as u8) <= MAX_LEVEL.load(Ordering::Relaxed)
}

/// Initialize the level threshold from `MONSTER_LOG` exactly once.
pub fn init_from_env() {
    if INIT_DONE.set(()).is_ok() {
        if let Ok(v) = std::env::var("MONSTER_LOG") {
            if let Some(level) = Level::parse(&v) {
                set_max_level(level);
            }
        }
    }
}

#[doc(hidden)]
pub fn write(level: Level, args: std::fmt::Arguments<'_>) {
    if should_log(level) {
        eprintln!("[MONSTER {:>5}] {}", level.as_str(), args);
    }
}

/// Log at `error` level — always visible in release builds.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Error, format_args!($($arg)*)) };
}

/// Log at `warn` level.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Warn, format_args!($($arg)*)) };
}

/// Log at `info` level (default threshold).
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Info, format_args!($($arg)*)) };
}

/// Log at `debug` level.
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Debug, format_args!($($arg)*)) };
}

/// Log at `trace` level.
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Trace, format_args!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_gating_and_parsing() {
        set_max_level(Level::Warn);
        assert!(should_log(Level::Error));
        assert!(should_log(Level::Warn));
        assert!(!should_log(Level::Info));
        assert!(!should_log(Level::Trace));
        assert_eq!(Level::parse("INFO"), Some(Level::Info));
        assert_eq!(Level::parse(" nope "), None);
        // restore a sane default for other tests in the binary
        set_max_level(Level::Info);
    }
}
