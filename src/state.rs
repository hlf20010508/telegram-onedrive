/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{client::telegram_bot::TelegramBotClient, env::Env};

pub struct State {
    pub env: Env,
    pub telegram_bot: TelegramBotClient,
    pub should_auto_delete: Mutex<bool>,
}

impl State {
    pub async fn new() -> Self {
        let env = Env::new();
        let telegram_bot = TelegramBotClient::new(&env).await.unwrap();
        let should_auto_delete = Mutex::new(env.should_auto_delete);

        Self {
            env,
            telegram_bot,
            should_auto_delete,
        }
    }
}

pub type AppState = Arc<State>;
