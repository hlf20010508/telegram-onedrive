/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{client::TelegramClient, message::TelegramMessage};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use grammers_client::types::PackedChat;
use proc_macros::{add_context, add_trace};
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    inner: Box<InnerError>,
}

pub type RawError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
struct InnerError {
    message: String,
    contexts: Vec<String>,
    details: Vec<String>,
    raw: Option<RawError>,
}

impl Error {
    pub fn new<T>(message: T) -> Self
    where
        T: Display,
    {
        Self {
            inner: Box::new(InnerError {
                message: message.to_string(),
                contexts: Vec::new(),
                details: Vec::new(),
                raw: None,
            }),
        }
    }

    pub fn raw<E>(mut self, e: E) -> Self
    where
        E: Into<RawError>,
    {
        self.inner.raw = Some(Into::into(e));

        self
    }

    pub fn details<T>(mut self, detail: T) -> Self
    where
        T: Display,
    {
        self.inner.details.push(detail.to_string());

        self
    }

    pub fn context<T>(mut self, context: T) -> Self
    where
        T: Display,
    {
        self.inner.contexts.insert(0, context.to_string());

        self
    }

    pub fn trace(&self) {
        tracing::error!("{}", self);
    }

    #[add_context]
    #[add_trace]
    pub async fn send(self, message: TelegramMessage) -> Result<Self> {
        message.reply(self.to_string()).await.details(&self)?;

        Ok(self)
    }

    #[add_context]
    #[add_trace]
    pub async fn send_chat<C>(self, telegram_client: &TelegramClient, chat: C) -> Result<Self>
    where
        C: Into<PackedChat>,
    {
        telegram_client
            .send_message(chat, self.to_string())
            .await
            .details(&self)?;

        Ok(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_fmt(
            f: &mut std::fmt::Formatter<'_>,
            mut base: String,
            contexts: &[String],
            details: &[String],
        ) -> std::fmt::Result {
            if !contexts.is_empty() {
                let contexts = contexts.join("\n- ");
                base.push_str(&format!("\ncontexts:\n- {}", contexts));
            }

            if !details.is_empty() {
                let details = details.join("\n--------\n");
                base.push_str(&format!("\ndetails:\n{}", details));
            }

            write!(f, "{}", base)
        }

        let message = &self.inner.message;
        let contexts = &self.inner.contexts;
        let details = &self.inner.details;

        let base = match &self.inner.raw {
            Some(e) => format!("{}: {}", message, e),
            None => message.clone(),
        };

        write_fmt(f, base, contexts, details)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultExt<T> {
    fn details<U>(self, message_details: U) -> Result<T>
    where
        U: Display;

    fn context<U>(self, message_context: U) -> Result<T>
    where
        U: Display;

    fn trace(self);

    fn unwrap_or_trace(self) -> T;
}

impl<T> ResultExt<T> for Result<T> {
    fn details<U>(self, message_details: U) -> Self
    where
        U: Display,
    {
        self.map_err(|e| e.details(message_details))
    }

    fn context<U>(self, message_context: U) -> Self
    where
        U: Display,
    {
        self.map_err(|e| e.context(message_context))
    }

    fn trace(self) {
        if let Err(e) = self {
            e.trace();
        }
    }

    fn unwrap_or_trace(self) -> T {
        self.unwrap_or_else(|e| {
            e.trace();
            panic!("{}", e);
        })
    }
}

pub trait ResultUnwrapExt {
    fn unwrap_both(self) -> Error;
}

impl ResultUnwrapExt for Result<Error> {
    fn unwrap_both(self) -> Error {
        self.unwrap_or_else(|e| e)
    }
}
