/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::visitor::{MessageVisitor, MetaVisitor};
use ansi_term::Color;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{
        self, format,
        time::{ChronoLocal, FormatTime},
        FormatEvent, FormatFields,
    },
    registry::LookupSpan,
};

pub struct EventFormatter;

impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &fmt::FmtContext<'_, S, N>,
        writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut message_visitor = MessageVisitor::default();
        event.record(&mut message_visitor);
        let message = message_visitor.message;

        write_message(writer, event, message)
    }
}

pub fn write_message(
    mut writer: format::Writer<'_>,
    event: &tracing::Event<'_>,
    message: String,
) -> std::fmt::Result {
    let level = event.metadata().level();

    write_time(&mut writer)?;
    write_meta(&mut writer, event)?;
    write!(writer, "{:>5} ", level)?;
    write_colored_message(&mut writer, *level, message)?;

    writeln!(writer)
}

fn write_time(writer: &mut format::Writer<'_>) -> std::fmt::Result {
    if ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f ".to_string())
        .format_time(writer)
        .is_err()
    {
        write!(writer, "Time error ")?;
    }

    Ok(())
}

fn write_colored_message(
    writer: &mut format::Writer<'_>,
    level: Level,
    mut message: String,
) -> std::fmt::Result {
    match level {
        tracing::Level::ERROR => message = Color::Red.paint(message).to_string(),
        tracing::Level::WARN => message = Color::Yellow.paint(message).to_string(),
        tracing::Level::INFO => message = Color::Green.paint(message).to_string(),
        tracing::Level::DEBUG => message = Color::Blue.paint(message).to_string(),
        tracing::Level::TRACE => {
            if message.contains("->") {
                message = Color::Purple.paint(message).to_string();
            } else if message.contains("<-") {
                message = Color::Cyan.paint(message).to_string();
            } else {
                message = Color::Yellow.paint(message).to_string();
            }
        }
    }

    write!(writer, "{}", message)
}

fn write_meta(writer: &mut format::Writer<'_>, event: &tracing::Event<'_>) -> std::fmt::Result {
    let mut meta_visitor = MetaVisitor::default();
    event.record(&mut meta_visitor);

    let module_path = event
        .metadata()
        .module_path()
        .map_or_else(|| meta_visitor.module_path, |s| s.to_string());

    let file = event
        .metadata()
        .file()
        .map_or_else(|| meta_visitor.file, |s| s.to_string());

    let line = event
        .metadata()
        .line()
        .map_or_else(|| meta_visitor.line, |u| u.to_string());

    writeln!(writer, " {} {}:{}", module_path, file, line)
}
