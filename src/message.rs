/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::grammers_tl_types as tl;
use grammers_client::types::{Chat, InputMessage, Media, Message};
use std::sync::Arc;

use crate::error::{Error, Result};

#[derive(Clone)]
pub struct TelegramMessage {
    raw: Arc<Message>,
}

impl TelegramMessage {
    pub fn new(message: Message) -> Self {
        Self {
            raw: Arc::new(message),
        }
    }

    pub fn chat(&self) -> Chat {
        self.raw.chat()
    }

    pub fn text(&self) -> &str {
        self.raw.text()
    }

    pub fn id(&self) -> i32 {
        self.raw.id()
    }

    pub fn media(&self) -> Option<Media> {
        self.raw.media()
    }

    pub fn sender(&self) -> Option<Chat> {
        self.raw.sender()
    }

    pub async fn respond<M: Into<InputMessage>>(&self, message: M) -> Result<Self> {
        let message_raw = self
            .raw
            .respond(message)
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to respond message"))?;

        Ok(Self::new(message_raw))
    }

    pub async fn reply<M: Into<InputMessage>>(&self, message: M) -> Result<Self> {
        let message_raw = self
            .raw
            .reply(message)
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to reply message"))?;

        Ok(Self::new(message_raw))
    }

    pub async fn delete(&self) -> Result<()> {
        self.raw
            .delete()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to delete message"))
    }

    pub fn forward_header(&self) -> Option<tl::enums::MessageFwdHeader> {
        self.raw.forward_header()
    }
}
