/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::time::Duration;

use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Chat, InputMessage, PackedChat};
use grammers_client::Update;
use tokio::sync::mpsc;

use super::TelegramClient;
use crate::error::{Error, Result, ResultExt};
use crate::message::{QueuedMessage, QueuedMessageType, TelegramMessage};

impl TelegramClient {
    pub async fn get_message<C>(&self, chat: C, message_id: i32) -> Result<TelegramMessage>
    where
        C: Into<PackedChat>,
    {
        let message_raw = self
            .client()
            .get_messages_by_id(chat, &[message_id])
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get message by id"))?
            .get(0)
            .ok_or_else(|| Error::new("message vec is empty"))?
            .to_owned()
            .ok_or_else(|| Error::new("message not found"))?;

        let message = TelegramMessage::new(self.clone(), message_raw);

        Ok(message)
    }

    pub async fn get_chat(&self, message: TelegramMessage) -> Result<Chat> {
        let mut dialogs = self.client().iter_dialogs();

        let chat_old = message.chat();

        while let Some(dialog) = dialogs
            .next()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get dialog"))?
        {
            let chat_new = dialog.chat();
            if chat_new.id() == chat_old.id() {
                return Ok(chat_new.to_owned());
            }
        }

        Err(Error::new("chat not found"))
    }

    pub fn iter_messages<C: Into<PackedChat>>(&self, chat: C) -> MessageIter {
        self.client().iter_messages(chat)
    }

    pub async fn delete_messages<C: Into<PackedChat>>(
        &self,
        chat: C,
        message_ids: &[i32],
    ) -> Result<usize> {
        self.client()
            .delete_messages(chat, message_ids)
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to delete messages"))
    }

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
            .ok_or_else(|| Error::new("failed to receive message result"))
            .context("respond")??
            .ok_or_else(|| Error::new("received message is None").context("respond"))
    }

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
            .ok_or_else(|| Error::new("failed to receive message result"))
            .context("edit")??;

        Ok(())
    }

    pub async fn next_update(&self) -> Result<Option<Update>> {
        self.client()
            .next_update()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "Failed to get next update"))
    }

    pub async fn push_queued_message(&self, queued_message: QueuedMessage) {
        self.message_queue().lock().await.push_back(queued_message);
    }

    pub async fn run_message_loop(&self) {
        let message_queue = self.message_queue();
        let telegram_client = self.clone();

        tokio::spawn(async move {
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
                                .client()
                                .send_message(chat, input_message)
                                .await
                                .map_err(|e| {
                                    Error::new_telegram_invocation(e, "failed to respond message")
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
                                .client()
                                .send_message(chat, input_message.reply_to(Some(message_id)))
                                .await
                                .map_err(|e| {
                                    Error::new_telegram_invocation(e, "failed to respond message")
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
                                .client()
                                .edit_message(chat, message_id, input_message)
                                .await
                                .map_err(|e| {
                                    Error::new_telegram_invocation(e, "failed to respond message")
                                });

                            match result {
                                Ok(_) => Ok(None),
                                Err(e) => Err(e),
                            }
                        }
                    };

                    tx.send(message_result).await.unwrap();
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }
}
