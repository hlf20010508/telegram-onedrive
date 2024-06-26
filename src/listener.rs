/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use futures::future::BoxFuture;
use futures::FutureExt;
use grammers_client::types::Message;
use grammers_client::Update;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::client::telegram_bot::TelegramBotClient;
use crate::error::{Error, Result};
use crate::extractor::{Extractor, Handler};

type EventFn = dyn Fn(Message, Arc<TelegramBotClient>) -> BoxFuture<'static, Result<()>>;

pub struct Listener<T> {
    pub telegram_bot: Arc<TelegramBotClient>,
    pub events: HashMap<String, Box<EventFn>>,
    pub state: Option<Arc<Mutex<T>>>,
}

impl<T> Listener<T> {
    pub fn new(telegram_bot: TelegramBotClient) -> Self {
        Self {
            telegram_bot: Arc::new(telegram_bot),
            events: HashMap::new(),
            state: None,
        }
    }

    pub fn on<'a, F, Args>(mut self, event_type: EventType, callback: &'static F) -> Self
    where
        F: Handler<Args>,
        Args: Extractor,
    {
        self.events.insert(
            event_type.to_string(),
            Box::new(move |message, client| callback.handle(message, client).boxed()),
        );

        self
    }

    async fn trigger(&self, event_name: EventType, message: Message) -> Result<()> {
        if let Some(callback) = self.events.get(event_name.to_str()) {
            callback(message, self.telegram_bot.clone()).await?;
        }

        Ok(())
    }

    pub fn get_event_names(&self) -> Vec<EventType> {
        self.events
            .keys()
            .map(|name| EventType::from(name))
            .collect()
    }

    pub fn with_state(mut self, state: T) -> Self {
        self.state = Some(Arc::new(Mutex::new(state)));

        self
    }

    pub async fn run(&self) {
        loop {
            if let Some(update) = self.telegram_bot.client.next_update().await.unwrap() {
                match update {
                    Update::NewMessage(message) => {
                        self.handle_message(message).await.unwrap();
                    }
                    _ => Err(Error::new("Unsupported update type")).unwrap(),
                }
            }
        }
    }

    async fn handle_message(&self, message: Message) -> Result<()> {
        if let Some(_) = message.media() {
            self.handle_media(message).await?;
        } else {
            let text = message.text();

            if !text.is_empty() {
                if text.starts_with('/') {
                    self.handle_command(message).await?;
                } else {
                    self.handle_text(message).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&self, message: Message) -> Result<()> {
        let text = message.text();

        for event in self.get_event_names() {
            if text.starts_with(event.to_str()) {
                self.trigger(event, message).await?;
                break;
            }
        }

        Ok(())
    }

    async fn handle_text(&self, message: Message) -> Result<()> {
        self.trigger(EventType::Text, message).await
    }

    async fn handle_media(&self, message: Message) -> Result<()> {
        self.trigger(EventType::Media, message).await
    }
}

pub enum EventType {
    Command(String),
    Text,
    Media,
}

impl EventType {
    pub fn to_str(&self) -> &str {
        match self {
            EventType::Command(command) => command.as_str(),
            EventType::Text => "__TEXT__",
            EventType::Media => "__MEDIA__",
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Command(command) => write!(f, "{}", command),
            _ => write!(f, "{}", self.to_str()),
        }
    }
}

impl From<&String> for EventType {
    fn from(value: &String) -> Self {
        if value == EventType::Text.to_str() {
            EventType::Text
        } else if value == EventType::Media.to_str() {
            EventType::Media
        } else {
            EventType::Command(value.to_string())
        }
    }
}
