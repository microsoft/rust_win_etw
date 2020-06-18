//! Provides a `log::Log` implementation that sends events to Event Tracing for Windows (ETW).

#![no_std]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::useless_let_if_seq)]

extern crate alloc;
use alloc::string::ToString;
use core::sync::atomic::{AtomicBool, Ordering};
use win_etw_macros::trace_logging_provider;

#[trace_logging_provider(guid = "7f006a22-73fb-4c17-b1eb-0a3070f9f187")]
trait RustLogProvider {
    fn log(module_path: &str, file: &str, line: u32, message: &str);
}

/// Provides a `log::Log` implementation that sends events to Event Tracing for Windows (ETW).
pub struct TraceLogger {
    provider: RustLogProvider,
    log_module_path: AtomicBool,
    log_file_path: AtomicBool,
}

impl TraceLogger {
    /// Registers the `TraceLogger` with ETW.
    pub fn new() -> Result<Self, win_etw_provider::Error> {
        let provider = RustLogProvider::new();
        Ok(TraceLogger {
            provider,
            log_module_path: AtomicBool::new(true),
            log_file_path: AtomicBool::new(true),
        })
    }

    /// Controls whether Rust module paths are included in event records.
    /// The default is `true` (module paths are included).
    /// This is provided to give control over privacy and to control the size of event records.
    pub fn set_log_module_path(&self, value: bool) {
        self.log_module_path.store(value, Ordering::Release);
    }

    /// Controls whether source file names and line numbers are included in event
    /// records. The default is `true` (source file names and line numbers are included).
    /// This is provided to give control over privacy and to control the size of event records.
    pub fn set_log_file_path(&self, value: bool) {
        self.log_file_path.store(value, Ordering::Release);
    }

    /// Returns `true` if this logger will include Rust module paths in event records.
    pub fn log_module_path(&self) -> bool {
        self.log_module_path.load(Ordering::Acquire)
    }

    /// Returns `true` if this logger will include source file names and line numbers in event
    /// records.
    pub fn log_file_path(&self) -> bool {
        self.log_file_path.load(Ordering::Acquire)
    }
}

fn level_to_etw(level: log::Level) -> win_etw_provider::Level {
    match level {
        log::Level::Error => win_etw_provider::Level::ERROR,
        log::Level::Warn => win_etw_provider::Level::WARN,
        log::Level::Info => win_etw_provider::Level::INFO,
        log::Level::Debug => win_etw_provider::Level::VERBOSE,
        log::Level::Trace => win_etw_provider::Level(6),
    }
}

impl log::Log for TraceLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.provider
            .log_is_enabled(Some(level_to_etw(metadata.level())))
    }

    fn log(&self, record: &log::Record) {
        let module_path = if self.log_module_path() {
            record.module_path().unwrap_or("")
        } else {
            ""
        };

        let file_path;
        let file_line;
        if self.log_file_path() {
            file_path = record.file().unwrap_or("");
            file_line = record.line().unwrap_or(0);
        } else {
            file_path = "";
            file_line = 0;
        }

        let message = record.args().to_string();
        let metadata = record.metadata();

        let options = win_etw_provider::EventOptions {
            level: Some(level_to_etw(metadata.level())),
            ..Default::default()
        };
        self.provider
            .log(Some(&options), module_path, file_path, file_line, &message);
    }

    fn flush(&self) {}
}
