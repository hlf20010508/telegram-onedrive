/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::client::{OneDriveClient, TelegramBotClient, TelegramUserClient};
use crate::env::Env;

pub struct State {
    pub env: Env,
    pub telegram_bot: TelegramBotClient,
    pub telegram_user: TelegramUserClient,
    pub onedrive: OneDriveClient,
    pub should_auto_delete: AtomicBool,
}

impl State {
    pub async fn new() -> Self {
        let env = Env::new();
        let telegram_bot = TelegramBotClient::new(&env).await.unwrap();
        let telegram_user = TelegramUserClient::new(&env).await.unwrap();
        let onedrive = OneDriveClient::new(&env).await;
        let should_auto_delete = AtomicBool::new(env.should_auto_delete);

        Self {
            env,
            telegram_bot,
            telegram_user,
            onedrive,
            should_auto_delete,
        }
    }
}

pub type AppState = Arc<State>;
