/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod events;
mod handler;

use crate::{
    env::BYPASS_PREFIX,
    error::{Result, ResultExt, ResultUnwrapExt},
    message::{ChatEntity, TelegramMessage},
    state::{AppState, State},
    tasker::Tasker,
    trace::indenter,
};
use events::Events;
pub use events::{EventType, HashMapExt};
use grammers_client::Update;
use handler::Handler;
use proc_macros::{add_context, add_trace};
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
            indenter::set_file_indenter(indenter::Coroutine::Task, async {
                tasker.run().await;
            })
            .await;
        });

        let telegram_user = self.state.telegram_user.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            indenter::set_file_indenter(indenter::Coroutine::Listener, async {
                // this is not only used to catch Update::MessageDeleted, but also keep the user alive
                // cooperate with reconnection policy
                // if only loop update, connection will still be closed after a long time
                // if only reconnection policy, in current grammers version, it will block the client
                loop {
                    let update = telegram_user.next_update().await.unwrap_or_trace();
                    if let Update::MessageDeleted(messages_info) = update {
                        // abort the task if the related message is deleted
                        // bot can only catch deleted message immediately if it is sent by itself
                        // that's why we use user client to catch it instead of bot client
                        let mut task_aborters = state.task_session.aborters.lock().await;

                        // ignore the deletion in none-channel chat
                        if let Some(chat_id) = messages_info.channel_id() {
                            for message_id in messages_info.messages() {
                                if let Some((aborter, message_id_related)) =
                                    task_aborters.remove(&(chat_id, *message_id))
                                {
                                    aborter.abort();
                                    state
                                        .task_session
                                        .delete_task(aborter.id)
                                        .await
                                        .unwrap_or_trace();

                                    // if deleted message is the forwarded message, also delete the indicator message, vice versa
                                    if let Some(message_id_related) = message_id_related {
                                        let chat = telegram_user
                                            .get_chat(&ChatEntity::Id(chat_id))
                                            .await
                                            .unwrap_or_trace();

                                        telegram_user
                                            .delete_messages(chat, &[message_id_related])
                                            .await
                                            .unwrap_or_trace();

                                        task_aborters.remove(&(chat_id, message_id_related));
                                    }
                                } else {
                                    state
                                        .task_session
                                        .delete_task_from_message_id_if_exists(chat_id, *message_id)
                                        .await
                                        .unwrap_or_trace();
                                }
                            }
                        }
                    }
                }
            })
            .await;
        });

        loop {
            self.handle_message().await.trace();
        }
    }

    #[add_context]
    #[add_trace]
    async fn handle_message(&self) -> Result<()> {
        let client = &self.state.telegram_bot;

        let update = client.next_update().await?;
        if let Update::NewMessage(message_raw) = update {
            // bypass message that the bot sent itself, and message that starts with bypass prefix
            if !message_raw.outgoing() && !message_raw.text().starts_with(BYPASS_PREFIX) {
                let message = TelegramMessage::new(client.clone(), message_raw);

                let handler = Handler::new(&self.events, &self.state);
                if let Err(e) = handler.handle_message(message.clone()).await {
                    e.send(message).await.unwrap_both().trace();
                }
            }
        }

        Ok(())
    }
}
