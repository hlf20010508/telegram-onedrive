/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2025 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod progress;

use crate::{client::utils::chat_from_hex, error::ResultExt, state::AppState};
use anyhow::{Context, Result};
use grammers_client::InputMessage;
pub use progress::Progress;
use progress::{ChatRecord, ProgressItem};
use std::time::Duration;
use tokio::spawn;

pub struct Progresser {
    state: AppState,
}

impl Progresser {
    pub const fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn run(state: AppState) {
        let progresser = Self::new(state);

        spawn(async move {
            tracing::info!("progresser started");

            loop {
                progresser.handle_chat_tasks_progress().await.trace();

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }

    async fn handle_chat_tasks_progress(&self) -> Result<()> {
        let progress = &self.state.progress;

        for ProgressItem {
            current_length,
            total_length,
            chat_bot_hex,
            message_id,
            filename,
            ..
        } in progress.iter_item().await
        {
            let chat_bot = chat_from_hex(&chat_bot_hex)?;

            let s = &format!(
                "\n<a href=\"https://t.me/c/{}/{}\">{}</a>: {:.2}/{:.2}MB",
                chat_bot.id,
                message_id,
                filename,
                current_length as f64 / 1024. / 1024.,
                total_length as f64 / 1024. / 1024.
            );

            progress.add_to_current_response(&chat_bot_hex, s).await;
        }

        for (
            chat_bot_hex,
            ChatRecord {
                message_number,
                progress_message_id,
                current_response,
                last_response,
                chat_user_hex,
            },
        ) in progress.iter_record().await
        {
            let telegram_bot = self.state.telegram_bot.clone();
            let chat_bot = chat_from_hex(&chat_bot_hex)?;

            if message_number == 0 {
                progress.remove_record(&chat_bot_hex).await;

                telegram_bot
                    .delete_messages(chat_bot, &[progress_message_id.unwrap()])
                    .await?;

                continue;
            }

            let chat_pending_tasks_number = self
                .state
                .task_session
                .get_chat_pending_tasks_number(&chat_bot_hex)
                .await?;

            if chat_pending_tasks_number > 0 {
                let s = &format!("\n\n{} more tasks pending...", chat_pending_tasks_number);
                progress.add_to_current_response(&chat_bot_hex, s).await;
            }

            if let Some(progress_message_id) = progress_message_id {
                let telegram_user = self.state.telegram_user.clone();
                let chat_user = chat_from_hex(&chat_user_hex)?;

                let latest_message = telegram_user
                    .iter_messages(chat_user)
                    .limit(1)
                    .next()
                    .await
                    .context("failed to iter messages for latest message")?;

                if let Some(latest_message) = latest_message {
                    if latest_message.id() == progress_message_id {
                        if last_response != current_response {
                            telegram_bot
                                .edit_message(
                                    chat_bot,
                                    progress_message_id.to_owned(),
                                    InputMessage::html(current_response.as_str()),
                                )
                                .await
                                .context(current_response.clone())?;
                        }
                    } else {
                        telegram_bot
                            .delete_messages(chat_bot, &[progress_message_id.to_owned()])
                            .await?;

                        let message = telegram_bot
                            .send_message(chat_bot, InputMessage::html(current_response.as_str()))
                            .await
                            .context(current_response.clone())?;

                        progress
                            .update_progress_message_id(&chat_bot_hex, message.id())
                            .await;
                    }
                }
            } else {
                let message = telegram_bot
                    .send_message(chat_bot, InputMessage::html(current_response.as_str()))
                    .await
                    .context(current_response.clone())?;

                progress
                    .update_progress_message_id(&chat_bot_hex, message.id())
                    .await;
            }

            progress
                .update_last_response(&chat_bot_hex, &current_response)
                .await;
        }

        Ok(())
    }
}
