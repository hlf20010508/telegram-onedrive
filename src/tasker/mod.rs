/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod handlers;
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
use grammers_client::InputMessage;
use path_slash::PathBufExt;
pub use session::TaskSession;
use std::{path::Path, sync::Arc, time::Duration};
pub use tasks::{CmdType, InsertTask};
use tokio::{spawn, sync::Semaphore};

pub struct Tasker {
    state: AppState,
}

impl Tasker {
    pub const fn new(state: AppState) -> Self {
        Self { state }
    }

    fn session(&self) -> &TaskSession {
        &self.state.task_session
    }

    pub fn run(state: AppState) {
        let tasker = Self::new(state);

        spawn(async move {
            tracing::info!("tasker started");

            let handler_num = ENV.get().unwrap().task_handler_num;

            let semaphore = Arc::new(Semaphore::new(handler_num as usize));

            loop {
                tasker.handle_tasks(semaphore.clone()).await.trace();

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    async fn handle_tasks(&self, semaphore: Arc<Semaphore>) -> Result<()> {
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

            tokio::spawn(async move {
                let _permit = semaphore_clone
                    .acquire()
                    .await
                    .context("failed to acquire semaphore for task handler")
                    .unwrap_or_trace();

                if let Err(e) = handler_dispatch(task, message.clone(), state_clone).await {
                    e.send(message).await.unwrap_both().trace();
                }
            });
        }

        Ok(())
    }
}

async fn handler_dispatch(
    task: tasks::Model,
    message: TelegramMessage,
    state: AppState,
) -> Result<()> {
    let session = &state.task_session;
    let progress = &state.progress;
    let telegram_bot = &state.telegram_bot;
    let telegram_user = &state.telegram_user;

    session
        .set_task_status(task.id, tasks::TaskStatus::Started)
        .await?;

    progress
        .insert(
            task.id,
            task.total_length as u64,
            &task.chat_bot_hex,
            &task.chat_user_hex,
            task.message_id,
            &task.filename,
        )
        .await;

    let result = match task.cmd_type {
        CmdType::Url => {
            tracing::info!("handle url task");

            handlers::url::handler(task.clone(), state.clone()).await
        }
        CmdType::File | CmdType::Link => {
            tracing::info!("handle file or link task");

            handlers::file::handler(task.clone(), state.clone()).await
        }
    };

    let chat_id = message.chat().id();

    match result {
        Ok(()) => {
            session
                .set_task_status(task.id, tasks::TaskStatus::Completed)
                .await?;

            if task.auto_delete {
                let chat_bot = chat_from_hex(&task.chat_bot_hex)?;
                let chat_user = chat_from_hex(&task.chat_user_hex)?;

                telegram_bot
                    .delete_messages(chat_bot, &[task.message_indicator_id])
                    .await?;

                if state
                    .task_session
                    .is_last_task(chat_id, task.message_indicator_id)
                    .await?
                {
                    telegram_user
                        .delete_messages(chat_user, &[task.message_id])
                        .await?;
                }
            } else {
                handle_completed_task(task.clone(), state.clone()).await?;
            }
        }
        Err(e) => {
            e.send(message.clone()).await.unwrap_both().trace();

            session
                .set_task_status(task.id, tasks::TaskStatus::Failed)
                .await?;

            handle_failed_task(task.clone(), state.clone()).await?;
        }
    }

    session.delete_task(task.id).await?;
    progress.remove(task.id).await;

    Ok(())
}

async fn handle_completed_task(task: tasks::Model, state: AppState) -> Result<()> {
    let chat_bot = chat_from_hex(&task.chat_bot_hex)?;

    let file_path_raw = Path::new(&task.root_path).join(task.filename);
    let file_path = file_path_raw.to_slash_lossy();

    let telegram_bot = &state.telegram_bot;

    let message_indicator = telegram_bot
        .get_message(chat_bot, task.message_indicator_id)
        .await?;

    let response = format!(
        "{}\n\nDone.\nFile uploaded to {}\nSize {:.2}MB.",
        message_indicator.text(),
        file_path,
        task.total_length as f64 / 1024.0 / 1024.0
    );
    message_indicator
        .edit(task.message_indicator_id, InputMessage::html(&response))
        .await
        .context(response)?;

    Ok(())
}

async fn handle_failed_task(task: tasks::Model, state: AppState) -> Result<()> {
    let chat_bot = chat_from_hex(&task.chat_bot_hex)?;

    let telegram_bot = &state.telegram_bot;

    let message_indicator = telegram_bot
        .get_message(chat_bot, task.message_indicator_id)
        .await?;

    let response = format!("{}\n\nFailed.", message_indicator.text());
    message_indicator
        .edit(task.message_id, InputMessage::html(&response))
        .await
        .context(response)?;

    Ok(())
}
