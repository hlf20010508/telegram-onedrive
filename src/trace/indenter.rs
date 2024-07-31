/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::future::Future;
use std::sync::Mutex;
use tokio::task::futures::TaskLocalFuture;
use tokio::task_local;
use tracing::{Event, Subscriber};
use tracing_appender::rolling;
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::formatter::{write_colored_message, write_level, write_time};
use super::visitor::MessageVisitor;
use crate::env::LOGS_PATH;

pub enum Coroutine {
    Listener,
    AuthServer,
    Message,
    Progress,
    Task,
    TaskWorker(u8),
}

pub struct EventIndenter {
    coroutine: Coroutine,
    indent: Mutex<usize>,
}

task_local! {
    static EVENT_INDENTER: EventIndenter;
}

pub fn set_file_indenter<F>(coroutine: Coroutine, f: F) -> TaskLocalFuture<EventIndenter, F>
where
    F: Future,
{
    EVENT_INDENTER.scope(
        EventIndenter {
            coroutine,
            indent: std::sync::Mutex::new(0),
        },
        f,
    )
}

pub struct FileIndenterLayer;

impl FileIndenterLayer {
    fn write_fmt(
        writer: &mut format::Writer<'_>,
        event: &tracing::Event<'_>,
        message: String,
    ) -> std::fmt::Result {
        write_time(writer)?;
        write_level(writer, event)?;

        let level = event.metadata().level();

        write_colored_message(writer, *level, message)?;

        writeln!(writer)
    }
}

impl<S> Layer<S> for FileIndenterLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        EVENT_INDENTER.with(|indenter| {
            let mut visitor = MessageVisitor::default();
            event.record(&mut visitor);

            let mut indent = indenter.indent.lock().unwrap();

            if visitor.message.contains("<-") {
                *indent -= 1;
            }

            let mut indent_spaces = "-".repeat(*indent);

            if !visitor.message.contains("->") && !visitor.message.contains("<-") {
                indent_spaces.push_str("--|");
            }

            let log_message = if visitor.message.contains("<-") {
                let message = visitor.message.replace("<-", "");
                format!("<-{}{}", indent_spaces, message)
            } else {
                format!("{}{}", indent_spaces, visitor.message)
            };

            let writer = get_writer_for_coroutine(&indenter.coroutine);
            let mut fmt_writer = FmtWriter::new(writer);
            let mut writer = format::Writer::new(&mut fmt_writer);
            Self::write_fmt(&mut writer, event, log_message).unwrap();

            if visitor.message.contains("->") {
                *indent += 1;
            }
        });
    }
}

fn get_writer_for_coroutine(coroutine: &Coroutine) -> impl std::io::Write {
    // TODO: date
    match coroutine {
        Coroutine::Listener => rolling::never(LOGS_PATH, "listener.log"),
        Coroutine::AuthServer => rolling::never(LOGS_PATH, "auth_server.log"),
        Coroutine::Message => rolling::never(LOGS_PATH, "message.log"),
        Coroutine::Progress => rolling::never(LOGS_PATH, "progress.log"),
        Coroutine::Task => rolling::never(LOGS_PATH, "task.log"),
        Coroutine::TaskWorker(id) => rolling::never(LOGS_PATH, format!("task_worker_{}.log", id)),
    }
}

struct FmtWriter<W> {
    writer: W,
}

impl<W> FmtWriter<W>
where
    W: std::io::Write,
{
    fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> std::fmt::Write for FmtWriter<W>
where
    W: std::io::Write,
{
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.writer
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
}
