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

use crate::{
    client::utils::chat_from_hex,
    env::ENV,
    error::{Error, Result, ResultExt, ResultUnwrapExt},
    message::TelegramMessage,
    state::AppState,
    trace::indenter,
};
use proc_macros::add_context;
use progress::Progress;
pub use session::{TaskAborter, TaskSession};
use std::{sync::Arc, time::Duration};
pub use tasks::CmdType;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct Tasker {
    state: AppState,
    progress: Arc<Progress>,
}

impl Tasker {
    pub fn new(state: AppState) -> Self {
        let progress = Arc::new(Progress::new(state.clone()));

        Self { state, progress }
    }

    fn session(&self) -> &TaskSession {
        &self.state.task_session
    }

    pub async fn run(&self) {
        tracing::info!("tasker started");

        self.session().clear().await.unwrap_or_trace();

        let progress_clone = self.progress.clone();
        tokio::spawn(async move {
            indenter::set_file_indenter(indenter::Coroutine::Progress, async {
                progress_clone.run().await;
            })
            .await;
        });

        let handler_num = ENV.get().unwrap().task_handler_num;

        let semaphore = Arc::new(Semaphore::new(handler_num as usize));

        let mut handler_id = 0;

        loop {
            handler_id += 1;

            self.handle_tasks(semaphore.clone(), handler_id)
                .await
                .trace();

            if handler_id == handler_num {
                handler_id = 0;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    #[add_context]
    async fn handle_tasks(&self, semaphore: Arc<Semaphore>, handler_id: u8) -> Result<()> {
        let aborters = self.state.task_session.aborters.read().await;
        let task = self.session().fetch_task().await?;

        if let Some(task) = task {
            let chat = chat_from_hex(&task.chat_bot_hex)?;

            let message = self
                .state
                .telegram_bot
                .get_message(chat, task.message_id)
                .await?;

            let semaphore_clone = semaphore.clone();
            let state_clone = self.state.clone();
            let progress_clone = self.progress.clone();

            let cancellation_token = aborters
                .get(&(chat.id, task.message_id))
                .ok_or_else(|| {
                    Error::new("task aborter not found")
                        .context(format!("task worker {}", handler_id))
                })?
                .0
                .token
                .clone();

            tokio::spawn(async move {
                indenter::set_file_indenter(indenter::Coroutine::TaskWorker(handler_id), async {
                    async fn handler(
                        task: tasks::Model,
                        message: TelegramMessage,
                        progress: Arc<Progress>,
                        cancellation_token: CancellationToken,
                        state: AppState,
                    ) -> Result<()> {
                        let task_id = task.id;
                        let session = &state.task_session;

                        session
                            .set_task_status(task_id, tasks::TaskStatus::Started)
                            .await?;

                        let result = match task.cmd_type {
                            CmdType::Url => {
                                tracing::info!("handle url task");

                                handlers::url::handler(task, progress, state.clone()).await
                            }
                            CmdType::File | CmdType::Link => {
                                tracing::info!("handle file or link task");

                                handlers::file::handler(
                                    task,
                                    progress,
                                    cancellation_token,
                                    state.clone(),
                                )
                                .await
                            }
                        };

                        match result {
                            Ok(()) => {
                                session
                                    .set_task_status(task_id, tasks::TaskStatus::Completed)
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

                    let cancellation_token_clone = cancellation_token.clone();

                    let fut = async {
                        let _permit = semaphore_clone
                            .acquire()
                            .await
                            .map_err(|e| {
                                Error::new("failed to acquire semaphore for task handler").raw(e)
                            })
                            .unwrap_or_trace();

                        if let Err(e) = handler(
                            task,
                            message.clone(),
                            progress_clone,
                            cancellation_token,
                            state_clone,
                        )
                        .await
                        {
                            e.send(message).await.unwrap_both().trace();
                        }
                    };

                    tokio::select! {
                        () = fut => {}
                        () = cancellation_token_clone.cancelled() => {}
                    }
                })
                .await;
            });
        }

        Ok(())
    }
}
