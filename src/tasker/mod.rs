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

use proc_macros::{add_context, add_trace};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use progress::Progress;

pub use session::TaskSession;
pub use tasks::CmdType;

use crate::client::ext::chat_from_hex;
use crate::env::WORKER_NUM;
use crate::error::{Result, ResultUnwrapExt};
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::trace::indenter;

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
        tracing::info!("tasker started");

        self.session.clear().await.unwrap();

        let progress_clone = self.progress.clone();
        tokio::spawn(async move {
            indenter::set_file_indenter(indenter::Coroutine::Progress, async {
                progress_clone.run().await;
            })
            .await;
        });

        let semaphore = Arc::new(Semaphore::new(WORKER_NUM as usize));

        let mut handler_id = 0;

        loop {
            handler_id += 1;

            if let Err(e) = self.handle_tasks(semaphore.clone(), handler_id).await {
                e.trace();
            }

            if handler_id == WORKER_NUM {
                handler_id = 0;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    #[add_context]
    #[add_trace]
    async fn handle_tasks(&self, semaphore: Arc<Semaphore>, handler_id: u8) -> Result<()> {
        let task = self.session.fetch_task().await?;

        if let Some(task) = task {
            let chat = chat_from_hex(&task.chat_bot_hex)?;

            let message = self
                .state
                .telegram_bot
                .get_message(chat, task.message_id)
                .await?;

            let semaphore_clone = semaphore.clone();
            let state_clone = self.state.clone();
            let session_clone = self.session.clone();
            let progress_clone = self.progress.clone();

            macro_rules! handle_task {
                ($handler_type: ident) => {
                    tokio::spawn(async move {
                        indenter::set_file_indenter(
                            indenter::Coroutine::TaskWorker(handler_id),
                            async {
                                async fn handler(
                                    task: tasks::Model,
                                    message: TelegramMessage,
                                    session: Arc<TaskSession>,
                                    progress: Arc<Progress>,
                                    state: AppState,
                                ) -> Result<()> {
                                    let task_id = task.id;

                                    session
                                        .set_task_status(task_id, tasks::TaskStatus::Started)
                                        .await?;

                                    match handlers::$handler_type::handler(task, progress, state)
                                        .await
                                    {
                                        Ok(_) => {
                                            session
                                                .set_task_status(
                                                    task_id,
                                                    tasks::TaskStatus::Completed,
                                                )
                                                .await?;
                                        }
                                        Err(e) => {
                                            e.send(message.clone()).await.unwrap_both().trace();

                                            session
                                                .set_task_status(task_id, tasks::TaskStatus::Failed)
                                                .await?;
                                        }
                                    }

                                    Ok(())
                                }

                                let _permit = semaphore_clone.acquire().await.unwrap();

                                if let Err(e) = handler(
                                    task,
                                    message.clone(),
                                    session_clone,
                                    progress_clone,
                                    state_clone,
                                )
                                .await
                                {
                                    e.send(message).await.unwrap_both().trace();
                                }
                            },
                        )
                        .await;
                    });
                };
            }

            match task.cmd_type {
                CmdType::Url => {
                    handle_task!(url);
                }
                CmdType::File => {
                    handle_task!(file);
                }
                CmdType::Link => todo!(),
            }
        }

        Ok(())
    }
}
