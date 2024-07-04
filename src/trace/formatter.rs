/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use tracing::Subscriber;
use tracing_subscriber::fmt::time::{ChronoLocal, FormatTime};
use tracing_subscriber::fmt::{self, FormatFields};
use tracing_subscriber::fmt::{format, FormatEvent};
use tracing_subscriber::registry::LookupSpan;

pub struct EventFormatter;

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
        writeln!(writer)
    }
}
