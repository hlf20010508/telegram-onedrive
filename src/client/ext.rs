/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::{Chat, Message, PackedChat};
use std::sync::Arc;

use crate::error::{Error, Result};

pub trait TelegramExt {
    async fn get_message<C>(&self, chat: C, message_id: i32) -> Result<Message>
    where
        C: Into<PackedChat>;

    async fn get_chat(&self, message: Arc<Message>) -> Result<Option<Chat>>;
}

impl TelegramExt for grammers_client::Client {
    async fn get_message<C>(&self, chat: C, message_id: i32) -> Result<Message>
    where
        C: Into<PackedChat>,
    {
        let message = self
            .get_messages_by_id(chat, &[message_id])
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get message by id"))?
            .get(0)
            .ok_or_else(|| Error::new("message vec is empty"))?
            .to_owned()
            .ok_or_else(|| Error::new("message not found"))?;

        Ok(message)
    }

    async fn get_chat(&self, message: Arc<Message>) -> Result<Option<Chat>> {
        let mut dialogs = self.iter_dialogs();

        let bot_chat = message.chat();

        while let Some(dialog) = dialogs
            .next()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get dialog"))?
        {
            let user_chat = dialog.chat();
            if user_chat.id() == bot_chat.id() {
                return Ok(Some(user_chat.to_owned()));
            }
        }

        Ok(None)
    }
}

pub fn chat_from_hex(chat_hex: &str) -> Result<PackedChat> {
    PackedChat::from_hex(chat_hex)
        .map_err(|e| Error::new_telegram_packed_chat("failed to parse chat hex to packed chat", e))
}
