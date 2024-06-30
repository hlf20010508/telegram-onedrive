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
mod macros;
mod state;
mod trace;
mod utils;

use handlers::{auth, auto_delete, clear, dir, help, logs, start};
use listener::{EventType, Listener};
use trace::trace_registor;

#[tokio::main]
async fn main() {
    let _worker_guard = trace_registor();

    Listener::new()
        .await
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
        .run()
        .await;
}
