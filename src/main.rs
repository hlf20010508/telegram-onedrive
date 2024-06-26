/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod client;
mod env;
mod error;
mod extractor;
mod listener;
mod models;

use client::telegram_bot::TelegramBotClient;
use env::Env;
use listener::Listener;
use models::State;

#[tokio::main]
async fn main() {
    let env = Env::new();
    let state = State { env };
    let telegram_bot_client = TelegramBotClient::new(&state.env).await.unwrap();

    let listener = Listener::new(telegram_bot_client).with_state(state);

    listener.run().await;
}
