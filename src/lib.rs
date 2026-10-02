mod accept;
#[cfg(target_os = "android")]
mod android_init;
mod endpoint;
mod error;
mod key;
mod net;
mod path;
mod relay;
mod services;
mod ticket;
mod watch;

use tracing_subscriber::filter::LevelFilter;

pub use self::{
    accept::*, endpoint::*, error::*, key::*, net::*, path::*, relay::*, services::*, ticket::*,
    watch::*,
};

uniffi::setup_scaffolding!();

/// The logging level. See the rust (log crate)[https://docs.rs/log] for more information.
#[derive(Debug, uniffi::Enum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> LevelFilter {
        match level {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off => LevelFilter::OFF,
        }
    }
}

/// Receives formatted Rust log lines (KeepTalking fork). Called from Rust
/// threads, so implementations must be thread-safe.
#[uniffi::export(with_foreign)]
pub trait LogSink: Send + Sync + std::fmt::Debug {
    fn line(&self, line: String);
}

struct SinkWriter(std::sync::Arc<dyn LogSink>);

impl std::io::Write for SinkWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for line in String::from_utf8_lossy(buf).lines() {
            if !line.trim().is_empty() {
                self.0.line(line.to_string());
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct SinkMakeWriter(std::sync::Arc<dyn LogSink>);

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SinkMakeWriter {
    type Writer = SinkWriter;

    fn make_writer(&'a self) -> Self::Writer {
        SinkWriter(self.0.clone())
    }
}

/// Route Rust `tracing` output, filtered by an `EnvFilter` directive string
/// (e.g. `"iroh_ble_transport=debug,blew=debug,warn"`), to `sink` — and to
/// stderr when `stderr` is true. Installs the process-wide subscriber, so
/// only the first call (of this or `set_log_level`) takes effect; returns
/// whether this one did.
#[uniffi::export]
pub fn set_log_sink(directives: String, sink: std::sync::Arc<dyn LogSink>, stderr: bool) -> bool {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};
    let Ok(filter) = EnvFilter::try_new(&directives) else {
        return false;
    };
    let sink_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .without_time()
        .with_writer(SinkMakeWriter(sink));
    let stderr_layer = stderr.then(|| fmt::layer().with_ansi(false).with_writer(std::io::stderr));
    tracing_subscriber::registry()
        .with(filter)
        .with(sink_layer)
        .with(stderr_layer)
        .try_init()
        .is_ok()
}

/// Set the logging level. Installs the process-wide subscriber, so it has no
/// effect once one is installed (by this or `set_log_sink`).
#[uniffi::export]
pub fn set_log_level(level: LogLevel) {
    use tracing_subscriber::{fmt, prelude::*, reload};
    let filter: LevelFilter = level.into();
    let (filter, _) = reload::Layer::new(filter);
    let mut layer = fmt::Layer::default();
    layer.set_ansi(false);
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .try_init();
}
