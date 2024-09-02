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

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use formatter::EventFormatter;
use indenter::FileIndenterLayer;

use crate::env::Env;

pub fn trace_registor() {
    let trace_level = &Env::new().trace_level;

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .event_format(EventFormatter::new(true));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(FileIndenterLayer)
        .with(
            EnvFilter::from_default_env().add_directive(
                format!("telegram_onedrive={}", trace_level)
                    .parse()
                    .unwrap(),
            ),
        )
        .init();

    cleaner::run();
}
