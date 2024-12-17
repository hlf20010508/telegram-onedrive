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

use env::{Env, ENV};
use handlers::{
    auth, auto_delete, clear, dir, drive, file, help, link, links, logs, start, url, version,
};
use listener::{EventType, HashMapExt, Listener};
use std::collections::HashMap;
use trace::trace_registor;

// tested on ubuntu server, 2C2G,
// if not using current_thread, invokes in grammers may be blocked for 1 minute
// but works fine on MacOS
#[tokio::main(flavor = "current_thread")]
async fn main() {
    ENV.get_or_init(Env::new);

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
        .on(EventType::command(version::PATTERN), version::handler)
        .on(EventType::media(), file::handler)
        .on(EventType::text(), link::handler);

    Listener::new(events).await.run().await;
}
