/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::{ChronoLocal, FormatTime};
use tracing_subscriber::fmt::{self, FormatFields};
use tracing_subscriber::fmt::{format, FormatEvent};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

struct EventFormatter;

impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        if let Err(_) = ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()).format_time(&mut writer) {
            write!(writer, "Time error")?;
            writeln!(writer)?;
        }

        writeln!(writer)?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)?;
        writeln!(writer)
    }
}

pub fn trace_registor() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::never(".", "log.txt");
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
