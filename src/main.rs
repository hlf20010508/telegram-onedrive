/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod auth_server;
mod client;
mod env;
mod error;
mod handlers;
mod listener;
mod message;
mod state;
mod tasker;
mod trace;
mod utils;

use std::collections::HashMap;

use handlers::{auth, auto_delete, clear, dir, drive, file, help, link, links, logs, start, url};
use listener::{EventType, HashMapExt, Listener};
use trace::{indenter, trace_registor};

#[tokio::main]
async fn main() {
    trace_registor();

    let events = HashMap::new()
        .on(EventType::command(start::PATTERN), start::handler)
        .on(EventType::command(help::PATTERN), help::handler)
        .on(
            EventType::command(auto_delete::PATTERN),
            auto_delete::handler,
        )
        .on(EventType::command(logs::PATTERN), logs::handler)
        .on(EventType::command(auth::PATTERN), auth::handler)
        .on(EventType::command(clear::PATTERN), clear::handler)
        .on(EventType::command(dir::PATTERN), dir::handler)
        .on(EventType::command(drive::PATTERN), drive::handler)
        .on(EventType::command(url::PATTERN), url::handler)
        .on(EventType::command(links::PATTERN), links::handler)
        .on(EventType::media(), file::handler)
        .on(EventType::text(), link::handler);

    indenter::set_file_indenter(indenter::Coroutine::Listener, async {
        Listener::new(events).await.run().await;
    })
    .await;
}
