/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{client::telegram_bot::TelegramBotClient, env::Env};

pub struct State {
    pub env: Env,
    pub telegram_bot: TelegramBotClient,
}

impl State {
    pub async fn new() -> Self {
        let env = Env::new();
        let telegram_bot = TelegramBotClient::new(&env).await.unwrap();

        Self { env, telegram_bot }
    }
}
