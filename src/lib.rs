#![deny(clippy::all, missing_docs)]
//! The logging level can be changed by the `RUST_LOG` environment variable just like
//! [`env_logger`].
//!
//! ## Example
//!
//! ```
//! use log::{trace, debug, info, warn, error};
//!
//! fil_logger::init();
//!
//! trace!("some tracing");
//! debug!("debug information");
//! info!("normal information");
//! warn!("a warning");
//! error!("error!");
//! ```
//!
//! [env_logger]: https://crates.io/crates/env_logger
mod single_file_writer;

use std::env;
use std::fs::File;

use flexi_logger::{self, DeferredNow, FormatFunction, LogTarget, Record};
use log::Level;
pub use single_file_writer::SingleFileWriter;

/// Logs in the same JSON format as [IPFS go-log] does.
///
/// One log entry has this structure:
///
/// ```json
/// {
///   "level": "<log-level>",
///   "ts":"<timestamp>",
///   "logger":"<module-name>",
///   "caller":"<source-file>:<line-number>",
///   "msg":"<log-message>"
/// }
/// ```
///
/// [IPFS go-log]: https://github.com/ipfs/go-log
pub fn go_log_json_format(
    writer: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = match record.level() {
        Level::Error => "error",
        Level::Warn => "warn",
        Level::Info => "info",
        Level::Debug => "debug",
        Level::Trace => "trace",
    };
    write!(
        writer,
        r#"{{"level":"{}","ts":"{}","logger":"{}","caller":"{}:{}","msg":"{}"}}""#,
        level,
        now.now().format("%Y-%m-%dT%H:%M:%S%.3f%:z"),
        record.module_path().unwrap_or("<unnamed>"),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        &record.args()
    )
}

/// Logs in the same console format as [IPFS go-log] does.
///
/// One log entry has this structure:
///
/// ```text
/// <timestamp>\t<log-level>\t<module-name>\t<source-file>:<line-number>\t<log-message>
/// ```
///
/// [IPFS go-log]: https://github.com/ipfs/go-log
pub fn go_log_console_format(
    writer: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        writer,
        "{}\t{}\t{}\t{}:{}\t{}",
        now.now().format("%Y-%m-%dT%H:%M:%S%.3f%:z"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        &record.args()
    )
}

/// Initializes a new logger. It logs to stderr.
///
/// The default format is:
///
/// ```text
/// <timestamp>\t<log-level>\t<module-name>\t<source-file>:<line-number>\t<log-message>
/// ```
///
/// If the environment variable `GOLOG_LOG_FMT=json` is set, then the output is formatted as JSON:
///
/// ```json
/// {
///   "level": "<log-level>",
///   "ts":"<timestamp>",
///   "logger":"<module-name>",
///   "caller":"<source-file>:<line-number>",
///   "msg":"<log-message>"
/// }
/// ```
///
/// # Panics
///
/// Panics if a global logger was already set.
pub fn init() {
    flexi_logger::Logger::with_env()
        .format(log_format())
        .start()
        .expect("Initializing logger failed. Was another logger already initialized?");
}

/// initializes a new logger that logs to an already opened [`std::fs::File`].
///
/// If the environment variable `GOLOG_LOG_FMT=json` is set, then the output is formatted as JSON.
///
/// # Panics
///
/// Panics if a global logger was already set.
///
/// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
pub fn init_with_file(file: File) {
    flexi_logger::Logger::with_env()
        .log_target(LogTarget::Writer(Box::new(SingleFileWriter::new(file))))
        .format(log_format())
        .start()
        .expect("Initializing logger failed. Was another logger already initialized?");
}

/// The log format is based on the `GOLOG_LOG_FMT` environment variable. It can be set to `json`.
fn log_format() -> FormatFunction {
    match env::var("GOLOG_LOG_FMT") {
        Ok(ref format) if format == "json" => go_log_json_format,
        _ => go_log_console_format,
    }
}