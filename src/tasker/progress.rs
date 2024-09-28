/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;
use path_slash::PathBufExt;
use proc_macros::{add_context, add_trace};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::session::{ChatHex, ChatTasks};
use super::{tasks, TaskSession};
use crate::client::ext::chat_from_hex;
use crate::error::{Error, Result, ResultUnwrapExt};
use crate::state::AppState;

pub struct Progress {
    session: Arc<TaskSession>,
    state: AppState,
    last_progress_response: Arc<Mutex<String>>,
}

impl Progress {
    pub fn new(state: AppState) -> Self {
        let session = state.task_session.clone();
        let last_progress_response = Arc::new(Mutex::new(String::new()));

        Self {
            session,
            state,
            last_progress_response,
        }
    }

    #[add_context]
    #[add_trace]
    pub async fn set_current_length(&self, id: i64, current_length: u64) -> Result<()> {
        self.session.set_current_length(id, current_length).await
    }

    pub async fn run(&self) {
        tracing::info!("progress started");

        let mut chat_progress_message_id = HashMap::new();

        loop {
            if let Err(e) = self
                .handle_chat_tasks_progress(&mut chat_progress_message_id)
                .await
            {
                e.trace();
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    #[add_context]
    async fn handle_chat_tasks_progress(
        &self,
        chat_progress_message_id: &mut HashMap<String, Option<i32>>,
    ) -> Result<()> {
        let chat_tasks = self.session.get_chats_tasks().await?;

        for (
            ChatHex {
                chat_bot_hex,
                chat_user_hex,
            },
            ChatTasks {
                current_tasks,
                completed_tasks,
                failed_tasks,
            },
        ) in chat_tasks
        {
            if chat_progress_message_id.get(&chat_bot_hex).is_none() {
                chat_progress_message_id.insert(chat_bot_hex.clone(), None);
            }

            self.handle_chat_current_tasks(
                current_tasks,
                &chat_bot_hex,
                &chat_user_hex,
                chat_progress_message_id,
            )
            .await?;

            self.handle_chat_completed_tasks(completed_tasks, &chat_user_hex)
                .await?;

            self.handle_chat_failed_tasks(failed_tasks, &chat_bot_hex, &chat_user_hex)
                .await?;
        }

        self.remove_chats_without_tasks(chat_progress_message_id)
            .await?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    async fn handle_chat_current_tasks(
        &self,
        current_tasks: Vec<tasks::Model>,
        chat_bot_hex: &str,
        chat_user_hex: &str,
        chat_progress_message_id: &mut HashMap<String, Option<i32>>,
    ) -> Result<()> {
        let telegram_bot = &self.state.telegram_bot;

        if !current_tasks.is_empty() {
            let result = self
                .sync_chat_progress(
                    chat_bot_hex,
                    chat_user_hex,
                    current_tasks,
                    chat_progress_message_id,
                )
                .await;

            if let Err(e) = result {
                let chat = chat_from_hex(chat_bot_hex)?;

                e.send_chat(telegram_bot, chat).await.unwrap_both().trace();
            }
        }

        Ok(())
    }

    #[add_context]
    #[add_trace]
    async fn handle_chat_completed_tasks(
        &self,
        completed_tasks: Vec<tasks::Model>,
        chat_user_hex: &str,
    ) -> Result<()> {
        let telegram_bot = &self.state.telegram_bot;
        let telegram_user = &self.state.telegram_user;

        let should_auto_delete = self.state.should_auto_delete.load(Ordering::Acquire);

        for task in completed_tasks {
            let chat = chat_from_hex(chat_user_hex)?;

            let message_user = telegram_user.get_message(chat, task.message_id).await?;

            if should_auto_delete {
                message_user.delete().await?;
            } else {
                let file_path_raw = Path::new(&task.root_path).join(task.filename);
                let file_path = file_path_raw.to_slash_lossy();

                let response = format!(
                    "{}\n\nDone.\nFile uploaded to {}\nSize {:.2}MB.",
                    message_user.text(),
                    file_path,
                    task.total_length as f64 / 1024.0 / 1024.0
                );
                if let Err(e) = telegram_user
                    .edit_message(chat, task.message_id, response.as_str())
                    .await
                    .details(response)
                {
                    let chat = chat_from_hex(chat_user_hex)?;

                    e.send_chat(telegram_bot, chat).await.unwrap_both().trace();
                }
            }

            self.session.delete_task(task.id).await?;
        }

        Ok(())
    }

    #[add_context]
    #[add_trace]
    async fn handle_chat_failed_tasks(
        &self,
        failed_tasks: Vec<tasks::Model>,
        chat_bot_hex: &str,
        chat_user_hex: &str,
    ) -> Result<()> {
        let telegram_bot = &self.state.telegram_bot;
        let telegram_user = &self.state.telegram_user;

        for task in failed_tasks {
            let chat = chat_from_hex(chat_user_hex)?;

            let message_user = telegram_user.get_message(chat, task.message_id).await?;

            let response = format!("{}\n\nFailed.", message_user.text());
            if let Err(e) = telegram_user
                .edit_message(chat, task.message_id, response.as_str())
                .await
                .details(response)
            {
                let chat = chat_from_hex(chat_bot_hex)?;

                e.send_chat(telegram_bot, chat).await.unwrap_both().trace();
            }

            self.session.delete_task(task.id).await?;
        }

        Ok(())
    }

    #[add_context]
    async fn remove_chats_without_tasks(
        &self,
        chat_progress_message_id: &mut HashMap<String, Option<i32>>,
    ) -> Result<()> {
        let telegram_bot = &self.state.telegram_bot;

        let mut chat_to_be_removed = Vec::new();

        for (chat_bot_hex, progress_message_id) in chat_progress_message_id.iter() {
            let has_started_tasks = self
                .session
                .does_chat_has_started_tasks(chat_bot_hex)
                .await?;

            if !has_started_tasks {
                let chat = chat_from_hex(chat_bot_hex)?;

                if let Some(progress_message_id) = progress_message_id {
                    if let Err(e) = telegram_bot
                        .delete_messages(chat, &[progress_message_id.to_owned()])
                        .await
                    {
                        e.send_chat(telegram_bot, chat).await.unwrap_both().trace();
                    }
                }

                tracing::debug!("chat without tasks to be removed: {}", chat.id);

                chat_to_be_removed.push(chat_bot_hex.clone());
            }
        }

        for chat_bot_hex in chat_to_be_removed {
            chat_progress_message_id.remove(&chat_bot_hex);
        }

        Ok(())
    }

    #[add_context]
    #[add_trace]
    async fn sync_chat_progress(
        &self,
        chat_bot_hex: &str,
        chat_user_hex: &str,
        current_tasks: Vec<tasks::Model>,
        chat_progress_message_id: &mut HashMap<String, Option<i32>>,
    ) -> Result<()> {
        let telegram_bot = &self.state.telegram_bot;
        let telegram_user = &self.state.telegram_user;

        let chat = chat_from_hex(chat_bot_hex)?;

        let mut response = "Progress:\n".to_string();

        for task_progress in current_tasks {
            response += &format!(
                "\n<a href=\"https://t.me/c/{}/{}\">{}</a>: {:.2}/{:.2}MB",
                chat.id,
                task_progress.message_id,
                task_progress.filename,
                task_progress.current_length as f64 / 1024. / 1024.,
                task_progress.total_length as f64 / 1024. / 1024.
            );
        }

        let pending_tasks_number = self
            .session
            .get_chat_pending_tasks_number(chat_bot_hex)
            .await?;

        if pending_tasks_number > 0 {
            response += &format!("\n\n{} more tasks pending...", pending_tasks_number);
        }

        let progress_message_id = chat_progress_message_id
            .get_mut(chat_bot_hex)
            .ok_or_else(|| Error::new("chat_bot_hex not in chat_progress_message_id"))?;

        let mut last_progress_response = self.last_progress_response.lock().await;

        match progress_message_id {
            Some(progress_message_id) => {
                let chat_user = chat_from_hex(chat_user_hex)?;

                let latest_message = telegram_user
                    .iter_messages(chat_user)
                    .limit(1)
                    .next()
                    .await
                    .map_err(|e| Error::new("failed to iter messages for latest message").raw(e))?;

                if let Some(latest_message) = latest_message {
                    if latest_message.id() == *progress_message_id {
                        if *last_progress_response != response {
                            telegram_bot
                                .edit_message(
                                    chat,
                                    progress_message_id.to_owned(),
                                    InputMessage::html(response.as_str()),
                                )
                                .await
                                .details(&response)?;
                        }
                    } else {
                        telegram_bot
                            .delete_messages(chat, &[progress_message_id.to_owned()])
                            .await?;

                        let message = telegram_bot
                            .send_message(chat, InputMessage::html(response.as_str()))
                            .await
                            .details(&response)?;

                        *progress_message_id = message.id();
                    }
                }
            }
            None => {
                let message = telegram_bot
                    .send_message(chat, InputMessage::html(response.as_str()))
                    .await
                    .details(&response)?;

                *progress_message_id = Some(message.id());
            }
        }

        *last_progress_response = response;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn update_filename(&self, id: i64, filename: &str) -> Result<()> {
        self.session.update_filename(id, filename).await
    }
}
