/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod formatter;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use formatter::EventFormatter;

use crate::env::LOG_PATH;

pub fn trace_registor() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::never(".", LOG_PATH);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .event_format(EventFormatter);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .event_format(EventFormatter);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(
            EnvFilter::from_default_env()
                .add_directive(format!("telegram_onedrive={}", "debug").parse().unwrap()),
        )
        .init();

    // worker guard must be returned, or the file appender will be dropped and nothing will be written in file
    guard
}
