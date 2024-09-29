/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    client::{OneDriveClient, TelegramClient},
    env::ENV,
    error::ResultExt,
    tasker::TaskSession,
};
use std::sync::{atomic::AtomicBool, Arc};

pub struct State {
    pub telegram_bot: TelegramClient,
    pub telegram_user: TelegramClient,
    pub onedrive: OneDriveClient,
    pub should_auto_delete: AtomicBool,
    pub task_session: Arc<TaskSession>,
}

impl State {
    pub async fn new() -> Self {
        let env = ENV.get().unwrap();

        let telegram_bot = TelegramClient::new_bot().await.unwrap_or_trace();
        let telegram_user = TelegramClient::new_user().await.unwrap_or_trace();
        let onedrive = OneDriveClient::new().await.unwrap_or_trace();
        let should_auto_delete = AtomicBool::new(env.should_auto_delete);
        let task_session = Arc::new(
            TaskSession::new(&env.tasker_session_path)
                .await
                .unwrap_or_trace(),
        );

        Self {
            telegram_bot,
            telegram_user,
            onedrive,
            should_auto_delete,
            task_session,
        }
    }
}

pub type AppState = Arc<State>;
