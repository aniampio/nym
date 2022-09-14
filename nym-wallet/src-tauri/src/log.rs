use std::str::FromStr;

use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::Manager;

pub fn setup_logging(app_handle: tauri::AppHandle) -> Result<(), log::SetLoggerError> {
    let dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message,
            ))
        })
        .level(global_level())
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("tokio_reactor", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("mio", log::LevelFilter::Warn)
        .level_for("want", log::LevelFilter::Warn)
        .level_for("sled", log::LevelFilter::Warn)
        .level_for("tungstenite", log::LevelFilter::Warn)
        .level_for("tokio_tungstenite", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Warn)
        .level_for("tokio_util", log::LevelFilter::Warn);

    let dispatch = if ::std::env::var("RUST_TRACE_OPERATIONS").is_ok() {
        dispatch.level_for("nym_wallet::operations", log::LevelFilter::Trace)
    } else {
        dispatch
    };

    dispatch
        .chain(std::io::stdout())
        .chain(fern::Output::call(move |record| {
            let msg = LogMessage {
                message: record.args().to_string(),
                level: record.level().into(),
            };
            app_handle.emit_all("log://log", msg).unwrap();
        }))
        .apply()
}

fn global_level() -> log::LevelFilter {
    if let Ok(s) = ::std::env::var("RUST_LOG") {
        log::LevelFilter::from_str(&s).unwrap_or(log::LevelFilter::Info)
    } else {
        log::LevelFilter::Info
    }
}

#[derive(Debug, Serialize, Clone)]
struct LogMessage {
    message: String,
    level: LogLevel,
}

// Serialize to u16 instead of strings.
#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
enum LogLevel {
    Trace = 1,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Trace => LogLevel::Trace,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Info => LogLevel::Info,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Error => LogLevel::Error,
        }
    }
}