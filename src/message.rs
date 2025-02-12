/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client::TelegramClient;
use anyhow::Result;
use grammers_client::types::{Chat, InputMessage, Media, Message, PackedChat};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct TelegramMessage {
    pub raw: Arc<Message>,
    client: TelegramClient,
    text_override: Option<String>,
}

impl TelegramMessage {
    pub fn new(client: TelegramClient, message: Message) -> Self {
        Self {
            raw: Arc::new(message),
            client,
            text_override: None,
        }
    }

    pub fn override_text(&mut self, text: String) {
        self.text_override = Some(text);
    }

    pub fn chat(&self) -> Chat {
        self.raw.chat()
    }

    pub fn text(&self) -> String {
        self.text_override
            .clone()
            .unwrap_or_else(|| self.raw.html_text())
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
        self.client.send_message(self.chat(), message).await
    }

    pub async fn reply<M: Into<InputMessage>>(&self, message: M) -> Result<Self> {
        self.client
            .reply_message(self.chat(), self.id(), message)
            .await
    }

    pub async fn edit<M: Into<InputMessage>>(&self, message_id: i32, new_message: M) -> Result<()> {
        self.client
            .edit_message(self.chat(), message_id, new_message)
            .await
    }
}

pub struct QueuedMessage {
    pub message_type: QueuedMessageType,
    pub input_message: InputMessage,
    pub chat: PackedChat,
    pub tx: Sender<Result<Option<TelegramMessage>>>,
}

impl QueuedMessage {
    pub fn new<M: Into<InputMessage>, C: Into<PackedChat>>(
        message_type: QueuedMessageType,
        input_message: M,
        chat: C,
        tx: Sender<Result<Option<TelegramMessage>>>,
    ) -> Self {
        Self {
            message_type,
            input_message: input_message.into(),
            chat: chat.into(),
            tx,
        }
    }
}

pub enum QueuedMessageType {
    Respond,
    // need to specify the target message id
    Reply(i32),
    Edit(i32),
}

#[derive(Clone)]
pub enum ChatEntity {
    Chat(Chat),
    Id(i64),
    Username(String),
}

impl From<Chat> for ChatEntity {
    fn from(chat: Chat) -> Self {
        Self::Chat(chat)
    }
}

impl From<i64> for ChatEntity {
    fn from(id: i64) -> Self {
        Self::Id(id)
    }
}

impl From<String> for ChatEntity {
    fn from(username: String) -> Self {
        Self::Username(username)
    }
}

pub struct MessageInfo {
    pub chat_entity: ChatEntity,
    pub id: i32,
}

impl MessageInfo {
    pub const fn new(chat_entity: ChatEntity, id: i32) -> Self {
        Self { chat_entity, id }
    }
}
