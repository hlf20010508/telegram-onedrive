/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Chat, InputMessage, Message, PackedChat};
use grammers_client::Update;

use super::TelegramClient;
use crate::error::{Error, Result};
use crate::message::TelegramMessage;

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

        let message = TelegramMessage::new(message_raw);

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
    ) -> Result<Message> {
        self.client()
            .send_message(chat, message)
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to respond message"))
    }

    pub async fn edit_message<C: Into<PackedChat>, M: Into<InputMessage>>(
        &self,
        chat: C,
        message_id: i32,
        new_message: M,
    ) -> Result<()> {
        self.client()
            .edit_message(chat, message_id, new_message)
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to edit message"))
    }

    pub async fn next_update(&self) -> Result<Option<Update>> {
        self.client()
            .next_update()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "Failed to get next update"))
    }
}
