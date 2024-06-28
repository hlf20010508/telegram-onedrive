/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::error::{Error, Result};
use crate::state::{AppState, State};
use futures::future::BoxFuture;
use futures::Future;
use futures::FutureExt;
use grammers_client::types::Message;
use grammers_client::Update;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

type EventFn = dyn Fn(Arc<Message>, AppState) -> BoxFuture<'static, Result<()>>;

pub struct Listener {
    pub events: HashMap<String, Box<EventFn>>,
    pub state: AppState,
}

impl Listener {
    pub async fn new() -> Self {
        let state = Arc::new(State::new().await);

        Self {
            events: HashMap::new(),
            state,
        }
    }

    pub fn on<F, Fut>(mut self, event_type: EventType, callback: F) -> Self
    where
        F: Fn(Arc<Message>, Arc<State>) -> Fut + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let boxed_callback = Box::new(move |message, state| callback(message, state).boxed());

        self.events.insert(event_type.to_string(), boxed_callback);

        self
    }

    async fn trigger(&self, event_name: EventType, message: Arc<Message>) -> Result<()> {
        if let Some(callback) = self.events.get(event_name.to_str()) {
            callback(message, self.state.clone()).await?;
        }

        Ok(())
    }

    pub fn get_event_names(&self) -> Vec<EventType> {
        self.events
            .keys()
            .map(|name| EventType::from(name))
            .collect()
    }

    pub async fn run(&self) {
        loop {
            let result = match self.state.telegram_bot.client.next_update().await {
                Ok(update) => match update {
                    Some(update) => match update {
                        Update::NewMessage(message) if !message.outgoing() => {
                            let message = Arc::new(message);
                            match self.handle_message(message.clone()).await {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e.send(message).await.unwrap()),
                            }
                        }
                        _ => Ok(()),
                    },
                    None => Ok(()),
                },
                Err(e) => Err(Error::context(e, "Failed to get next update")),
            };

            if let Err(e) = result {
                e.trace();
            }
        }
    }

    async fn handle_message(&self, message: Arc<Message>) -> Result<()> {
        match message.media() {
            Some(_) => self.handle_media(message).await?,
            None => {
                let text = message.text();

                if !text.is_empty() {
                    if text.starts_with('/') {
                        self.handle_command(message).await?;
                    } else {
                        self.handle_text(message).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&self, message: Arc<Message>) -> Result<()> {
        let text = message.text();

        for event in self.get_event_names() {
            if text.starts_with(event.to_str()) {
                self.trigger(event, message).await?;
                break;
            }
        }

        Ok(())
    }

    async fn handle_text(&self, message: Arc<Message>) -> Result<()> {
        self.trigger(EventType::Text, message).await
    }

    async fn handle_media(&self, message: Arc<Message>) -> Result<()> {
        self.trigger(EventType::Media, message).await
    }
}

pub enum EventType {
    Command(String),
    Text,
    Media,
}

impl EventType {
    pub fn command(pattern: &str) -> Self {
        EventType::Command(pattern.to_string())
    }

    pub fn text() -> Self {
        EventType::Text
    }

    pub fn media() -> Self {
        EventType::Media
    }

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
