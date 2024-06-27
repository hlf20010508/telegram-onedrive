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

use listener::Listener;

#[tokio::main]
async fn main() {
    let listener = Listener::new().await;

    listener.run().await;
}
