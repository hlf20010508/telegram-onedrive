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
    error::{ErrorExt, ResultExt, ResultUnwrapExt},
    message::TelegramMessage,
    state::AppState,
};
use anyhow::{Context, Result};
use progress::Progress;
pub use session::{TaskAborter, TaskSession};
use std::{sync::Arc, time::Duration};
pub use tasks::{CmdType, InsertTask};
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

        let progress_clone = self.progress.clone();
        tokio::spawn(async move {
            progress_clone.run().await;
        });

        let handler_num = ENV.get().unwrap().task_handler_num;

        let semaphore = Arc::new(Semaphore::new(handler_num as usize));

        loop {
            self.handle_tasks(semaphore.clone()).await.trace();

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn handle_tasks(&self, semaphore: Arc<Semaphore>) -> Result<()> {
        let mut aborters = self.state.task_session.aborters.lock().await;
        let task = self.session().fetch_task().await?;

        if let Some(task) = task {
            let chat = chat_from_hex(&task.chat_bot_hex)?;

            // in case that message is sent and deleted immediately
            let Ok(message) = self
                .state
                .telegram_bot
                .get_message(chat, task.message_id)
                .await
            else {
                self.state
                    .task_session
                    .delete_task_from_message_id_if_exists(chat.id, task.message_id)
                    .await?;

                tracing::info!("task {} aborted", task.filename);

                return Ok(());
            };

            let semaphore_clone = semaphore.clone();
            let state_clone = self.state.clone();
            let progress_clone = self.progress.clone();

            let aborter = Arc::new(TaskAborter::new(task.id, &task.filename));
            let cancellation_token = aborter.token.clone();

            // insert both message_id and message_id_forward so that both of them can be used to abort the task
            aborters.insert(
                (chat.id, task.message_id),
                (aborter.clone(), task.message_id_forward),
            );

            if let Some(message_id_forward) = task.message_id_forward {
                aborters.insert(
                    (chat.id, message_id_forward),
                    (aborter, Some(task.message_id)),
                );
            }

            drop(aborters);

            tokio::spawn(async move {
                let cancellation_token_clone = cancellation_token.clone();

                let fut = async {
                    let _permit = semaphore_clone
                        .acquire()
                        .await
                        .context("failed to acquire semaphore for task handler")
                        .unwrap_or_trace();

                    if let Err(e) = handler_dispatch(
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
            });
        }

        Ok(())
    }
}

async fn handler_dispatch(
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

            handlers::url::handler(task, progress).await
        }
        CmdType::File | CmdType::Link => {
            tracing::info!("handle file or link task");

            handlers::file::handler(task, progress, cancellation_token, state.clone()).await
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

    let mut aborters = state.task_session.aborters.lock().await;
    let chat_id = message.chat().id();
    if let Some((_, Some(message_id_related))) = aborters.remove(&(chat_id, message.id())) {
        aborters.remove(&(chat_id, message_id_related));
    }

    Ok(())
}
