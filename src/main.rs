/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod client;
mod env;
mod error;
mod handlers;
mod listener;
mod state;
mod trace;

use handlers::{help, start};
use listener::{EventType, Listener};

#[tokio::main]
async fn main() {
    Listener::new()
        .await
        .on(EventType::command(start::PATTERN), start::handler)
        .on(EventType::command(help::PATTERN), help::handler)
        .run()
        .await;
}
