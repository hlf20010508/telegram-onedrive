/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod cleaner;
mod formatter;
pub mod indenter;
mod visitor;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use formatter::EventFormatter;
use indenter::FileIndenterLayer;

use crate::env::Env;

pub fn trace_registor() -> WorkerGuard {
    LogTracer::init().unwrap();

    let trace_level = &Env::new().trace_level;

    let stdout_layer = fmt::layer().with_writer(std::io::stdout);
    // .event_format(EventFormatter::new(true));

    let file_appender = tracing_appender::rolling::never(".", "dev.log.txt");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer().with_writer(file_writer);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        // .with(FileIndenterLayer)
        .with(
            // EnvFilter::from_default_env().add_directive(
            //     format!("telegram_onedrive={}", trace_level)
            //         .parse()
            //         .unwrap(),
            EnvFilter::from_default_env()
                .add_directive(format!("sqlx=error").parse().unwrap())
                .add_directive(format!("hyper_util=error").parse().unwrap())
                // .add_directive(
                //     format!("grammers_session::message_box=error")
                //         .parse()
                //         .unwrap(),
                // )
                .add_directive(format!("reqwest::connect=error").parse().unwrap())
                .add_directive(format!("telegram_onedrive=debug").parse().unwrap())
                .add_directive(format!("trace").parse().unwrap()),
        )
        .init();

    cleaner::run();

    guard
}
