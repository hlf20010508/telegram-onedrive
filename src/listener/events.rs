/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use futures::future::BoxFuture;
use futures::{Future, FutureExt};
use grammers_client::types::Message;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use crate::error::Result;
use crate::state::AppState;

type EventFn = dyn Fn(Arc<Message>, AppState) -> BoxFuture<'static, Result<()>> + Send + Sync;
pub type Events = HashMap<String, Box<EventFn>>;

pub trait HashMapExt {
    fn on<F, Fut>(self, event_type: EventType, callback: F) -> Self
    where
        F: Fn(Arc<Message>, AppState) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static;
}

impl HashMapExt for Events {
    fn on<F, Fut>(mut self, event_type: EventType, callback: F) -> Self
    where
        F: Fn(Arc<Message>, AppState) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let boxed_callback = Box::new(move |message, state| callback(message, state).boxed());

        self.insert(event_type.to_string(), boxed_callback);

        self
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
