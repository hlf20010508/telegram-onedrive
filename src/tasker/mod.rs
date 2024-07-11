/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod handlers;
mod progress;
mod session;
mod tasks;
mod transfer;
mod var;

use grammers_session::PackedChat;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use progress::Progress;

pub use session::TaskSession;
pub use tasks::CmdType;

use crate::client::ext::TelegramExt;
use crate::env::WORKER_NUM;
use crate::error::{Error, Result, ResultExt};
use crate::state::AppState;

pub struct Tasker {
    state: AppState,
    session: Arc<TaskSession>,
    progress: Arc<Progress>,
}

impl Tasker {
    pub async fn new(state: AppState) -> Result<Self> {
        let session = state.task_session.clone();
        let progress = Arc::new(Progress::new(state.clone()));

        Ok(Self {
            state,
            session,
            progress,
        })
    }

    pub async fn run(&self) {
        tracing::debug!("tasker started");

        self.session.prune_tasks().await.unwrap();

        let progress_clone = self.progress.clone();
        tokio::spawn(async move {
            progress_clone.run().await;
        });

        let semaphore = Arc::new(Semaphore::new(WORKER_NUM));

        loop {
            let result = match self.session.fetch_task().await {
                Ok(task) => match task {
                    Some(task) => match task.cmd_type {
                        CmdType::Url => {
                            match PackedChat::from_hex(&task.chat_bot_hex).map_err(|_| {
                                Error::new("failed to parse chat bot hex to packed chat")
                            }) {
                                Ok(chat) => {
                                    match self
                                        .state
                                        .telegram_bot
                                        .client
                                        .get_message(chat, task.message_id)
                                        .await
                                    {
                                        Ok(message) => {
                                            let message = Arc::new(message);

                                            match PackedChat::from_hex(&task.chat_user_hex).map_err(|_| {
                                                Error::new("failed to parse chat user hex to packed chat")
                                            }) {
                                                Ok(chat) => {
                                                    match self.state.telegram_user.client.get_message(chat, task.message_id).await {
                                                        Ok(message_user) => {
                                                            let semaphore_clone = semaphore.clone();
                                                            let session_clone = self.session.clone();
                                                            let progress_clone = self.progress.clone();

                                                            tokio::spawn(async move {
                                                                let _permit =
                                                                    semaphore_clone.acquire().await.unwrap();

                                                                let task_id = task.id;

                                                                match session_clone
                                                                    .set_task_status(
                                                                        task_id,
                                                                        tasks::TaskStatus::Started,
                                                                    )
                                                                    .await
                                                                {
                                                                    Ok(_) => {
                                                                        match handlers::url::handler(
                                                                            task,
                                                                            message_user,
                                                                            progress_clone,
                                                                        )
                                                                        .await
                                                                        {
                                                                            Ok(_) => {
                                                                                if let Err(e) = session_clone
                                                                                    .set_task_status(
                                                                                        task_id,
                                                                                        tasks::TaskStatus::Completed,
                                                                                    )
                                                                                    .await
                                                                                {
                                                                                    e.send(message).await.unwrap_both().trace()
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                e.send(message.clone())
                                                                                    .await
                                                                                    .unwrap_both()
                                                                                    .trace();

                                                                                if let Err(e) = session_clone
                                                                                    .set_task_status(
                                                                                        task_id,
                                                                                        tasks::TaskStatus::Failed,
                                                                                    )
                                                                                    .await
                                                                                {
                                                                                    e.send(message).await.unwrap_both().trace()
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        e.send(message).await.unwrap_both().trace()
                                                                    }
                                                                }
                                                            });
                                                            
                                                            Ok(())
                                                        },
                                                        Err(e) => Err(e),
                                                    }
                                                },
                                                Err(e) => Err(e),
                                            }
                                        }
                                        Err(e) => Err(e),
                                    }
                                }
                                Err(e) => Err(e),
                            }
                        }
                        CmdType::File => todo!(),
                        CmdType::Photo => todo!(),
                        CmdType::Link => todo!(),
                    },
                    None => Ok(()),
                },
                Err(e) => Err(e),
            };

            if let Err(e) = result {
                e.trace();
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
