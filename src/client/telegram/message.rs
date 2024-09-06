/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Chat, InputMessage, PackedChat};
use grammers_client::Update;
use proc_macros::{add_context, add_trace};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::sync::mpsc;

use super::TelegramClient;
use crate::error::{Error, Result};
use crate::message::{ChatEntity, QueuedMessage, QueuedMessageType, TelegramMessage};
use crate::trace::indenter;

impl TelegramClient {
    #[add_context]
    #[add_trace]
    pub async fn get_message<C>(&self, chat: C, message_id: i32) -> Result<TelegramMessage>
    where
        C: Into<PackedChat>,
    {
        let message_raw = self
            .raw()
            .get_messages_by_id(chat, &[message_id])
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get message by id"))?
            .first()
            .ok_or_else(|| Error::new("message vec is empty"))?
            .to_owned()
            .ok_or_else(|| Error::new("message not found"))?;

        let message = TelegramMessage::new(self.clone(), message_raw);

        Ok(message)
    }

    #[add_context]
    #[add_trace]
    pub async fn get_chat(&self, chat_entity: &ChatEntity) -> Result<Chat> {
        let mut dialogs = self.raw().iter_dialogs();

        while let Some(dialog) = dialogs
            .next()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get dialog"))?
        {
            let chat = dialog.chat();

            if match chat_entity {
                ChatEntity::Chat(chat_old) => chat.id() == chat_old.id(),
                ChatEntity::Id(chat_id) => chat.id() == *chat_id,
                ChatEntity::Username(chat_username) => match chat.username() {
                    Some(username) => username == chat_username,
                    None => chat.usernames().contains(&chat_username.as_str()),
                },
            } {
                return Ok(chat.to_owned());
            }
        }

        Err(Error::new("chat not found"))
    }

    #[add_trace]
    pub fn iter_messages<C: Into<PackedChat>>(&self, chat: C) -> MessageIter {
        self.raw().iter_messages(chat)
    }

    #[add_context]
    #[add_trace]
    pub async fn delete_messages<C: Into<PackedChat>>(
        &self,
        chat: C,
        message_ids: &[i32],
    ) -> Result<usize> {
        self.raw()
            .delete_messages(chat, message_ids)
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to delete messages"))
    }

    #[add_context]
    #[add_trace]
    pub async fn send_message<C: Into<PackedChat>, M: Into<InputMessage>>(
        &self,
        chat: C,
        message: M,
    ) -> Result<TelegramMessage> {
        let (tx, mut rx) = mpsc::channel(1);

        let queued_message = QueuedMessage::new(QueuedMessageType::Respond, message, chat, tx);

        self.push_queued_message(queued_message).await;

        rx.recv()
            .await
            .ok_or_else(|| Error::new("failed to receive message result"))??
            .ok_or_else(|| Error::new("received message is None"))
    }

    #[add_context]
    #[add_trace]
    pub async fn edit_message<C: Into<PackedChat>, M: Into<InputMessage>>(
        &self,
        chat: C,
        message_id: i32,
        new_message: M,
    ) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        let queued_message =
            QueuedMessage::new(QueuedMessageType::Edit(message_id), new_message, chat, tx);

        self.push_queued_message(queued_message).await;

        rx.recv()
            .await
            .ok_or_else(|| Error::new("failed to receive message result"))??;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn next_update(&self) -> Result<Update> {
        self.raw()
            .next_update()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "Failed to get next update"))
    }

    #[add_trace]
    pub async fn push_queued_message(&self, queued_message: QueuedMessage) {
        self.message_queue().lock().await.push_back(queued_message);
    }

    #[add_trace]
    pub fn run_message_loop(&self) {
        let message_queue = self.message_queue();
        let telegram_client = self.clone();

        let mut rng = {
            let rng = rand::thread_rng();
            StdRng::from_rng(rng).unwrap()
        };

        tokio::spawn(async move {
            indenter::set_file_indenter(indenter::Coroutine::Message, async {
                loop {
                    if let Some(QueuedMessage {
                        message_type,
                        input_message,
                        chat,
                        tx,
                    }) = message_queue.lock().await.pop_front()
                    {
                        let message_result = match message_type {
                            QueuedMessageType::Respond => {
                                let result = telegram_client
                                    .raw()
                                    .send_message(chat, input_message)
                                    .await
                                    .map_err(|e| {
                                        Error::new_telegram_invocation(
                                            e,
                                            "failed to respond message",
                                        )
                                    });

                                match result {
                                    Ok(message_raw) => Ok(Some(TelegramMessage::new(
                                        telegram_client.clone(),
                                        message_raw,
                                    ))),
                                    Err(e) => Err(e),
                                }
                            }
                            QueuedMessageType::Reply(message_id) => {
                                let result = telegram_client
                                    .raw()
                                    .send_message(chat, input_message.reply_to(Some(message_id)))
                                    .await
                                    .map_err(|e| {
                                        Error::new_telegram_invocation(
                                            e,
                                            "failed to respond message",
                                        )
                                    });

                                match result {
                                    Ok(message_raw) => Ok(Some(TelegramMessage::new(
                                        telegram_client.clone(),
                                        message_raw,
                                    ))),
                                    Err(e) => Err(e),
                                }
                            }
                            QueuedMessageType::Edit(message_id) => {
                                let result = telegram_client
                                    .raw()
                                    .edit_message(chat, message_id, input_message)
                                    .await
                                    .map_err(|e| {
                                        Error::new_telegram_invocation(
                                            e,
                                            "failed to respond message",
                                        )
                                    });

                                match result {
                                    Ok(()) => Ok(None),
                                    Err(e) => Err(e),
                                }
                            }
                        };

                        tx.send(message_result).await.unwrap();
                    }

                    let millis = rng.gen_range(1500..4000);
                    tokio::time::sleep(Duration::from_millis(millis)).await;
                }
            })
            .await;
        });
    }
}

pub struct MessageVecDeque {
    deque: VecDeque<QueuedMessage>,
    key_map: HashMap<i32, usize>,
}

impl MessageVecDeque {
    pub fn new() -> Self {
        Self {
            deque: VecDeque::new(),
            key_map: HashMap::new(),
        }
    }

    #[add_trace]
    pub fn push_back(&mut self, queued_message: QueuedMessage) {
        match queued_message.message_type {
            QueuedMessageType::Respond | QueuedMessageType::Reply(_) => {
                self.deque.push_back(queued_message);
            }
            QueuedMessageType::Edit(message_id) => match self.key_map.get(&message_id) {
                Some(index) => {
                    self.deque[*index] = queued_message;
                }
                None => {
                    self.key_map.insert(message_id, self.deque.len());
                    self.deque.push_back(queued_message);
                }
            },
        }
    }

    #[add_trace]
    pub fn pop_front(&mut self) -> Option<QueuedMessage> {
        self.deque.pop_front().map(|queued_message| {
            if let QueuedMessageType::Edit(message_id) = queued_message.message_type {
                self.key_map.remove(&message_id);
            }

            queued_message
        })
    }
}
