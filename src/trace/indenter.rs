/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    formatter::write_message,
    visitor::{MessageVisitor, MetaVisitor},
};
use crate::{env::LOGS_PATH, error::catch_unwind_silent};
use std::{future::Future, sync::Mutex};
use tokio::{task::futures::TaskLocalFuture, task_local};
use tracing::{Event, Subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::format, layer::Context, registry::LookupSpan, Layer};

// to seperate logs to different files

pub enum Coroutine {
    Listener,
    Message,
    Progress,
    Task,
    TaskWorker(u8),
}

// to better show the function call stack
// if enter the function, indent number + 1
// if leave the function, indent number - 1
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

impl<S> Layer<S> for FileIndenterLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let mut meta_visitor = MetaVisitor::default();
        event.record(&mut meta_visitor);
        let module_path = event
            .metadata()
            .module_path()
            .map_or_else(|| meta_visitor.module_path, |s| s.to_string());

        let (writer, log_message) = if module_path.starts_with("telegram_onedrive")
            && !module_path.starts_with("telegram_onedrive::auth_server")
        {
            catch_unwind_silent(|| {
                // only for events emitted by this project
                EVENT_INDENTER.with(|indenter| {
                    /*
                    example:
                    ->|func1
                    -->|func2
                    ---|some info1
                    --->|func3
                    ----|some info2
                    <---|func3
                    <--|func2
                    <-|func1
                    */
                    let mut indent = indenter.indent.lock().unwrap();

                    if visitor.message.contains("<-") {
                        *indent -= 1;
                    }

                    let mut indent_spaces = "-".repeat(*indent);

                    // other events that are not for showing the functions call stack
                    if !visitor.message.contains("->") && !visitor.message.contains("<-") {
                        indent_spaces.push_str("--|");
                    }

                    let log_message = if visitor.message.contains("<-") {
                        let message = visitor.message.replace("<-", "");
                        format!("<-{}{}", indent_spaces, message)
                    } else {
                        format!("{}{}", indent_spaces, visitor.message)
                    };

                    if visitor.message.contains("->") {
                        *indent += 1;
                    }

                    let writer = get_writer_for_coroutine(&indenter.coroutine);

                    (writer, log_message)
                })
            })
            .unwrap_or((log_builder("others"), visitor.message))
        } else {
            let writer = if module_path.starts_with("grammers") {
                log_builder("grammers")
            } else if module_path.starts_with("hyper_util") {
                log_builder("hyper_util")
            } else if module_path.starts_with("reqwest") {
                log_builder("reqwest")
            } else if module_path.starts_with("telegram_onedrive::auth_server") {
                // server event handler works in seperate coroutine
                // so local task doesn't work for it
                log_builder("auth_server")
            } else {
                log_builder("others")
            };

            (writer, visitor.message)
        };

        let mut fmt_writer = FmtWriter::new(writer);
        let writer = format::Writer::new(&mut fmt_writer);
        write_message(writer, event, log_message).unwrap();
    }
}

fn log_builder(name: &str) -> RollingFileAppender {
    RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix(format!("{}.log", name))
        .build(LOGS_PATH)
        .unwrap()
}

fn get_writer_for_coroutine(coroutine: &Coroutine) -> RollingFileAppender {
    match coroutine {
        Coroutine::Listener => log_builder("listener"),
        Coroutine::Message => log_builder("message"),
        Coroutine::Progress => log_builder("progress"),
        Coroutine::Task => log_builder("task"),
        Coroutine::TaskWorker(id) => log_builder(&format!("task_worker_{}", id)),
    }
}

struct FmtWriter<W> {
    writer: W,
}

impl<W> FmtWriter<W>
where
    W: std::io::Write,
{
    const fn new(writer: W) -> Self {
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
