/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod cleaner;
mod formatter;
mod visitor;

use crate::env::{ENV, LOGS_PATH};
use formatter::EventFormatter;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn trace_registor() {
    LogTracer::init().unwrap();

    let trace_level = &ENV.get().unwrap().trace_level;

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .event_format(EventFormatter);

    let file_layer = fmt::layer()
        .with_writer(log_writer_builder)
        .event_format(EventFormatter);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(EnvFilter::new(trace_level).add_directive("sqlx=error".parse().unwrap()))
        .init();

    cleaner::run();
}

fn log_writer_builder() -> RollingFileAppender {
    RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .build(LOGS_PATH)
        .unwrap()
}
