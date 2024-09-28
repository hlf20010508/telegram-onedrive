/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::client::{OneDriveClient, TelegramClient};
use crate::env::Env;
use crate::error::ResultExt;
use crate::tasker::TaskSession;

pub struct State {
    pub env: Env,
    pub telegram_bot: TelegramClient,
    pub telegram_user: TelegramClient,
    pub onedrive: OneDriveClient,
    pub should_auto_delete: AtomicBool,
    pub task_session: Arc<TaskSession>,
}

impl State {
    pub async fn new() -> Self {
        let env = Env::new();
        let telegram_bot = TelegramClient::new_bot(&env).await.unwrap_or_trace();
        let telegram_user = TelegramClient::new_user(&env).await.unwrap_or_trace();
        let onedrive = OneDriveClient::new(&env).await.unwrap_or_trace();
        let should_auto_delete = AtomicBool::new(env.should_auto_delete);
        let task_session = Arc::new(
            TaskSession::new(&env.tasker_session_path)
                .await
                .unwrap_or_trace(),
        );

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
