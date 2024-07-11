/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;
use grammers_session::PackedChat;
use path_slash::PathBufExt;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::{tasks, TaskSession};
use crate::client::ext::TelegramExt;
use crate::error::{Error, Result, ResultExt};
use crate::state::AppState;

pub struct Progress {
    session: Arc<TaskSession>,
    state: AppState,
}

impl Progress {
    pub fn new(state: AppState) -> Self {
        let session = state.task_session.clone();

        Self { session, state }
    }

    pub async fn set_current_length(&self, id: i64, current_length: u64) -> Result<()> {
        self.session.set_current_length(id, current_length).await
    }

    pub async fn run(&self) {
        tracing::debug!("progress started");

        let mut chat_progress_message_id = HashMap::new();

        let telegram_bot = &self.state.telegram_bot;
        let telegram_user = &self.state.telegram_user;

        loop {
            match self.session.get_chats_tasks().await {
                Ok(chats_tasks) => {
                    for (
                        (chat_bot_hex, chat_user_hex),
                        (current_tasks, completed_tasks, failed_tasks),
                    ) in chats_tasks
                    {
                        if chat_progress_message_id.get(&chat_bot_hex).is_none() {
                            chat_progress_message_id.insert(chat_bot_hex.clone(), None);
                        }

                        if !current_tasks.is_empty() {
                            let result = self
                                .sync_chat_progress(
                                    &chat_bot_hex,
                                    &chat_user_hex,
                                    &current_tasks,
                                    &mut chat_progress_message_id,
                                )
                                .await;

                            if let Err(e) = result {
                                match PackedChat::from_hex(&chat_bot_hex).map_err(|_| {
                                    Error::new("failed to parse chat bot hex to packed chat")
                                }) {
                                    Ok(chat) => {
                                        e.send_chat(&telegram_bot.client, chat)
                                            .await
                                            .unwrap_both()
                                            .trace();
                                    }
                                    Err(e) => e.trace(),
                                }
                            }
                        }

                        for task in completed_tasks {
                            match PackedChat::from_hex(&chat_user_hex).map_err(|_| {
                                Error::new("failed to parse chat user hex to packed chat")
                            }) {
                                Ok(chat) => {
                                    match telegram_user
                                        .client
                                        .get_message(chat, task.message_id)
                                        .await
                                    {
                                        Ok(message_user) => {
                                            let file_path_raw =
                                                Path::new(&task.root_path).join(task.filename);
                                            let file_path = file_path_raw.to_slash_lossy();

                                            let response = format!(
                                                "{}\n\nDone.\nFile uploaded to {}",
                                                message_user.text(),
                                                file_path
                                            );
                                            if let Err(e) = telegram_user
                                                .client
                                                .edit_message(
                                                    chat,
                                                    task.message_id,
                                                    response.as_str(),
                                                )
                                                .await
                                                .map_err(|e| Error::respond_error(e, response))
                                            {
                                                match PackedChat::from_hex(&chat_bot_hex).map_err(|_| {
                                                    Error::new("failed to parse chat bot hex to packed chat")
                                                }) {
                                                    Ok(chat) => {
                                                        e.send_chat(&telegram_bot.client, chat)
                                                            .await
                                                            .unwrap_both()
                                                            .trace();
                                                    }
                                                    Err(e) => e.trace(),
                                                }
                                            }
                                        }
                                        Err(e) => e.trace(),
                                    }
                                }
                                Err(e) => e.trace(),
                            }

                            if let Err(e) = self.session.delete_task(task.id).await {
                                e.trace();
                            }
                        }

                        for task in failed_tasks {
                            match PackedChat::from_hex(&chat_user_hex).map_err(|_| {
                                Error::new("failed to parse chat user hex to packed chat")
                            }) {
                                Ok(chat) => {
                                    match telegram_user
                                        .client
                                        .get_message(chat, task.message_id)
                                        .await
                                    {
                                        Ok(message_user) => {
                                            let response =
                                                format!("{}\n\nFailed.", message_user.text());
                                            if let Err(e) = telegram_user
                                                .client
                                                .edit_message(
                                                    chat,
                                                    task.message_id,
                                                    response.as_str(),
                                                )
                                                .await
                                                .map_err(|e| Error::respond_error(e, response))
                                            {
                                                match PackedChat::from_hex(&chat_bot_hex).map_err(|_| {
                                                    Error::new("failed to parse chat bot hex to packed chat")
                                                }) {
                                                    Ok(chat) => {
                                                        e.send_chat(&telegram_bot.client, chat)
                                                            .await
                                                            .unwrap_both()
                                                            .trace();
                                                    }
                                                    Err(e) => e.trace(),
                                                }
                                            }
                                        }
                                        Err(e) => e.trace(),
                                    }
                                }
                                Err(e) => e.trace(),
                            }

                            if let Err(e) = self.session.delete_task(task.id).await {
                                e.trace();
                            }
                        }
                    }

                    let mut chat_to_be_removed = Vec::new();

                    for (chat_bot_hex, progress_message_id) in &chat_progress_message_id {
                        match self
                            .session
                            .does_chat_has_started_tasks(&chat_bot_hex)
                            .await
                        {
                            Ok(has_started_tasks) => {
                                if !has_started_tasks {
                                    match PackedChat::from_hex(&chat_bot_hex).map_err(|_| {
                                        Error::new("failed to parse chat bot hex to packed chat")
                                    }) {
                                        Ok(chat) => {
                                            if let Some(progress_message_id) = progress_message_id {
                                                if let Err(e) = telegram_bot
                                                    .client
                                                    .delete_messages(
                                                        chat,
                                                        &[progress_message_id.to_owned()],
                                                    )
                                                    .await
                                                    .map_err(|e| {
                                                        Error::context(
                                                            e,
                                                            "failed to delete progress message",
                                                        )
                                                    })
                                                {
                                                    e.send_chat(&telegram_bot.client, chat)
                                                        .await
                                                        .unwrap_both()
                                                        .trace();
                                                }
                                            }
                                        }
                                        Err(e) => e.trace(),
                                    }

                                    chat_to_be_removed.push(chat_bot_hex.clone());
                                }
                            }
                            Err(e) => e.trace(),
                        }
                    }

                    for chat_bot_hex in chat_to_be_removed {
                        chat_progress_message_id.remove(&chat_bot_hex);
                    }
                }
                Err(e) => e.trace(),
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn sync_chat_progress(
        &self,
        chat_bot_hex: &str,
        chat_user_hex: &str,
        current_tasks: &Vec<tasks::Model>,
        chat_progress_message_id: &mut HashMap<String, Option<i32>>,
    ) -> Result<()> {
        let telegram_bot = &self.state.telegram_bot;
        let telegram_user = &self.state.telegram_user;

        let chat = PackedChat::from_hex(chat_bot_hex)
            .map_err(|_| Error::new("failed to parse chat bot hex to packed chat"))?;

        let mut response = "Progress:\n".to_string();

        for task_progress in current_tasks {
            response += &format!(
                "\n<a href=\"https://t.me/c/{}/{}\">{}</a>: {:.2}/{:.2}MB",
                chat.id,
                task_progress.message_id,
                task_progress.filename,
                task_progress.current_length / 1024 / 1024,
                task_progress.total_length / 1024 / 1024
            );
        }

        let pending_tasks_number = self
            .session
            .get_chat_pending_tasks_number(chat_bot_hex)
            .await?;

        if pending_tasks_number > 0 {
            response += &format!("\n\n{} more tasks pending...", pending_tasks_number);
        }

        let progress_message_id = chat_progress_message_id.get_mut(chat_bot_hex).unwrap();

        match progress_message_id {
            Some(progress_message_id) => {
                let chat_user = PackedChat::from_hex(chat_user_hex)
                    .map_err(|_| Error::new("failed to parse chat user hex to packed chat"))?;

                let latest_message = telegram_user
                    .client
                    .iter_messages(chat_user)
                    .limit(1)
                    .next()
                    .await
                    .map_err(|e| Error::context(e, "failed to iter messages for latest message"))?;

                if let Some(latest_message) = latest_message {
                    if latest_message.id() != progress_message_id.to_owned() {
                        telegram_bot
                            .client
                            .delete_messages(chat, &[progress_message_id.to_owned()])
                            .await
                            .map_err(|e| Error::context(e, "failed to delete progress message"))?;

                        let message = telegram_bot
                            .client
                            .send_message(chat, InputMessage::html(response.as_str()))
                            .await
                            .map_err(|e| Error::respond_error(e, response))?;

                        *progress_message_id = message.id();
                    } else {
                        if let Err(e) = telegram_bot
                            .client
                            .edit_message(
                                chat,
                                progress_message_id.to_owned(),
                                InputMessage::html(response.as_str()),
                            )
                            .await
                        {
                            match e {
                                grammers_client::client::bots::InvocationError::Rpc(e) => {
                                    if e.code != 400 {
                                        Err(Error::respond_error(e, response))?
                                    }
                                }
                                _ => Err(Error::respond_error(e, response))?,
                            }
                        }
                    }
                }
            }
            None => {
                let message = telegram_bot
                    .client
                    .send_message(chat, InputMessage::html(response.as_str()))
                    .await
                    .map_err(|e| Error::respond_error(e, response))?;

                *progress_message_id = Some(message.id());
            }
        }

        Ok(())
    }

    pub async fn update_filename(&self, id: i64, filename: &str) -> Result<()> {
        self.session.update_filename(id, filename).await
    }
}
