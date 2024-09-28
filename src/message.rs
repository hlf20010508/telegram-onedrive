/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::grammers_tl_types as tl;
use grammers_client::types::{Chat, InputMessage, Media, Message, PackedChat};
use proc_macros::{add_context, add_trace};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};

use crate::client::TelegramClient;
use crate::error::{Error, Result};

#[derive(Clone)]
pub struct TelegramMessage {
    pub raw: Arc<Message>,
    client: TelegramClient,
}

impl TelegramMessage {
    pub fn new(client: TelegramClient, message: Message) -> Self {
        Self {
            raw: Arc::new(message),
            client,
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

    #[add_context]
    #[add_trace]
    pub async fn respond<M: Into<InputMessage>>(&self, message: M) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel(1);

        let queued_message =
            QueuedMessage::new(QueuedMessageType::Respond, message, self.chat(), tx);

        self.client.push_queued_message(queued_message).await;

        rx.recv()
            .await
            .ok_or_else(|| Error::new("failed to receive message result"))??
            .ok_or_else(|| Error::new("received message is None"))
    }

    #[add_context]
    #[add_trace]
    pub async fn reply<M: Into<InputMessage>>(&self, message: M) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel(1);

        let queued_message = QueuedMessage::new(
            QueuedMessageType::Reply(self.id()),
            message,
            self.chat(),
            tx,
        );

        self.client.push_queued_message(queued_message).await;

        rx.recv()
            .await
            .ok_or_else(|| Error::new("failed to receive message result"))??
            .ok_or_else(|| Error::new("received message is None"))
    }

    #[add_context]
    #[add_trace]
    pub async fn delete(&self) -> Result<()> {
        self.raw
            .delete()
            .await
            .map_err(|e| Error::new("failed to delete message").raw(e))
    }

    pub fn forward_header(&self) -> Option<tl::enums::MessageFwdHeader> {
        self.raw.forward_header()
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
    pub fn new(chat_entity: ChatEntity, id: i32) -> Self {
        Self { chat_entity, id }
    }
}
