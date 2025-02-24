/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod events;
mod handler;

use crate::{
    client::utils::chat_from_hex,
    error::{ErrorExt, ResultExt, ResultUnwrapExt},
    message::{ChatEntity, TelegramMessage},
    state::{AppState, State},
    tasker::Tasker,
};
use anyhow::{Ok, Result};
use events::Events;
pub use events::{EventType, HashMapExt};
use grammers_client::Update;
use handler::Handler;
use std::sync::Arc;

pub struct Listener {
    pub events: Events,
    pub state: AppState,
}

impl Listener {
    pub async fn new(events: Events) -> Self {
        let state = Arc::new(State::new().await);

        Self { events, state }
    }

    pub async fn run(self) {
        tracing::info!("listener started");

        let tasker = Tasker::new(self.state.clone());
        tokio::spawn(async move {
            tasker.run().await;
        });

        let state = self.state.clone();
        tokio::spawn(async move {
            loop {
                handle_batch_cancellation(state.clone())
                    .await
                    .unwrap_or_trace();
            }
        });

        loop {
            self.handle_message().await.trace();
        }
    }

    async fn handle_message(&self) -> Result<()> {
        let client = &self.state.telegram_bot;
        let telegram_user = &self.state.telegram_user;
        let task_session = &self.state.task_session;

        let update = client.next_update().await?;
        match update {
            Update::NewMessage(message_raw) => {
                // bypass message that the bot sent itself
                if !message_raw.outgoing() {
                    let message = TelegramMessage::new(client.clone(), message_raw);

                    let handler = Handler::new(&self.events, self.state.clone());
                    if let Err(e) = handler.handle_message(message.clone()).await {
                        e.send(message).await.unwrap_both().trace();
                    }
                }
            }
            Update::MessageDeleted(messages_info) => {
                // abort the task if the related message is deleted
                // bot can only catch deleted message immediately if it is sent by itself
                let mut task_aborters = task_session.task_aborters.lock().await;

                // ignore the deletion in none-channel chat
                if let Some(chat_id) = messages_info.channel_id() {
                    for message_indicator_id in messages_info.messages() {
                        if let Some(task_aborter) =
                            task_aborters.remove(&(chat_id, *message_indicator_id))
                        {
                            task_aborter.abort();

                            let batch_aborters = task_session.batch_aborters.lock().await;
                            let batch_is_processing = batch_aborters
                                .get(&(chat_id, task_aborter.message_id))
                                .is_some_and(|batch_aborter| batch_aborter.processing);
                            drop(batch_aborters);

                            // must before deleting task
                            let should_delete_message = task_session
                                .is_last_task(chat_id, *message_indicator_id)
                                .await
                                .unwrap_or_trace()
                                && !batch_is_processing;

                            task_session
                                .delete_task(task_aborter.id)
                                .await
                                .unwrap_or_trace();

                            if should_delete_message {
                                let chat =
                                    chat_from_hex(&task_aborter.chat_user_hex).unwrap_or_trace();

                                telegram_user
                                    .delete_messages(chat, &[task_aborter.message_id])
                                    .await
                                    .unwrap_or_trace();
                            }
                        } else {
                            task_session
                                .delete_task_from_message_indicator_id_if_exists(
                                    chat_id,
                                    *message_indicator_id,
                                )
                                .await
                                .unwrap_or_trace();
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

async fn handle_batch_cancellation(state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let task_session = &state.task_session;
    let update = telegram_user.next_update().await?;

    if let Update::MessageDeleted(messages_info) = update {
        if let Some(chat_id) = messages_info.channel_id() {
            for message_id in messages_info.messages() {
                let mut batch_aborters = task_session.batch_aborters.lock().await;
                if let Some(batch_aborter) = batch_aborters.remove(&(chat_id, *message_id)) {
                    batch_aborter.abort();
                }
                drop(batch_aborters);

                let mut task_aborters = task_session.task_aborters.lock().await;
                let message_indicator_ids = task_session
                    .get_message_indicator_ids(chat_id, *message_id)
                    .await?;
                for message_indicator_id in message_indicator_ids {
                    let chat_user = if let Some(aborter) =
                        task_aborters.remove(&(chat_id, message_indicator_id))
                    {
                        aborter.abort();
                        task_session.delete_task(aborter.id).await?;

                        chat_from_hex(&aborter.chat_user_hex)?
                    } else {
                        task_session
                            .delete_task_from_message_indicator_id_if_exists(
                                chat_id,
                                message_indicator_id,
                            )
                            .await?;

                        telegram_user
                            .get_chat(&ChatEntity::from(chat_id))
                            .await?
                            .pack()
                    };

                    telegram_user
                        .delete_messages(chat_user, &[message_indicator_id])
                        .await?;
                }
            }
        }
    }

    Ok(())
}
