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
use crate::tasker::TaskSession;

pub struct State {
    pub env: Env,
    pub telegram_bot: TelegramBotClient,
    pub telegram_user: TelegramUserClient,
    pub onedrive: OneDriveClient,
    pub should_auto_delete: AtomicBool,
    pub task_session: Arc<TaskSession>,
}

impl State {
    pub async fn new() -> Self {
        let env = Env::new();
        let telegram_bot = TelegramBotClient::new(&env).await.unwrap();
        let telegram_user = TelegramUserClient::new(&env).await.unwrap();
        let onedrive = OneDriveClient::new(&env).await.unwrap();
        let should_auto_delete = AtomicBool::new(env.should_auto_delete);
        let task_session = Arc::new(TaskSession::new(&env.tasker_session_path).await.unwrap());

        Self {
            env,
            telegram_bot,
            telegram_user,
            onedrive,
            should_auto_delete,
            task_session,
        }
    }
}

pub type AppState = Arc<State>;
