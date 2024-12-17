/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{message::TelegramMessage, state::AppState};
use anyhow::Result;
use futures::{future::BoxFuture, Future, FutureExt};
use std::{collections::HashMap, fmt::Display};

type EventFn = dyn Fn(TelegramMessage, AppState) -> BoxFuture<'static, Result<()>>;
pub type Events = HashMap<String, Box<EventFn>>;

pub trait HashMapExt {
    fn on<F, Fut>(self, event_type: EventType, callback: F) -> Self
    where
        F: Fn(TelegramMessage, AppState) -> Fut + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static;
}

impl HashMapExt for Events {
    fn on<F, Fut>(mut self, event_type: EventType, callback: F) -> Self
    where
        F: Fn(TelegramMessage, AppState) -> Fut + 'static,
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
        Self::Command(pattern.to_string())
    }

    pub const fn text() -> Self {
        Self::Text
    }

    pub const fn media() -> Self {
        Self::Media
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Command(command) => command.as_str(),
            Self::Text => "__TEXT__",
            Self::Media => "__MEDIA__",
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(command) => write!(f, "{}", command),
            _ => write!(f, "{}", self.to_str()),
        }
    }
}

impl From<&String> for EventType {
    fn from(value: &String) -> Self {
        if value == Self::Text.to_str() {
            Self::Text
        } else if value == Self::Media.to_str() {
            Self::Media
        } else {
            Self::Command(value.to_string())
        }
    }
}
