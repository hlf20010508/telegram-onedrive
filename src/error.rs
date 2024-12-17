/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{client::TelegramClient, message::TelegramMessage};
use anyhow::{Context, Error, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use grammers_client::{types::PackedChat, InputMessage};
use std::{fmt::Display, panic};

pub trait ResultExt<T> {
    fn trace(self);

    fn unwrap_or_trace(self) -> T;
}

impl<T> ResultExt<T> for Result<T> {
    fn trace(self) {
        if let Err(e) = self {
            tracing::error!("{:?}", e);
        }
    }

    fn unwrap_or_trace(self) -> T {
        self.unwrap_or_else(|e| {
            tracing::error!("{:?}", e);
            panic!();
        })
    }
}

pub trait ErrorExt {
    fn trace(self);

    fn format_tg(&self) -> String;

    async fn send(self, message: TelegramMessage) -> Result<Error>;

    async fn send_chat<C>(self, telegram_client: &TelegramClient, chat: C) -> Result<Error>
    where
        C: Into<PackedChat>;
}

impl ErrorExt for Error {
    fn trace(self) {
        tracing::error!("{:?}", self);
    }

    fn format_tg(&self) -> String {
        let mut message = self.to_string();

        let chain = self.chain().skip(1);
        if chain.clone().count() > 0 {
            message.push_str("\nCaused by:");

            for cause in chain {
                message.push_str(&format!("\n- {}", cause));
            }
        }

        message
    }

    async fn send(self, message: TelegramMessage) -> Result<Self> {
        message
            .reply(InputMessage::html(self.format_tg()))
            .await
            .context(self.format_tg())?;

        Ok(self)
    }

    async fn send_chat<C>(self, telegram_client: &TelegramClient, chat: C) -> Result<Self>
    where
        C: Into<PackedChat>,
    {
        telegram_client
            .send_message(chat, self.format_tg())
            .await
            .context(self.format_tg())?;

        Ok(self)
    }
}

pub struct HttpError {
    raw: Box<dyn Display>,
}

impl HttpError {
    pub fn new<T: Display + 'static>(e: T) -> Self {
        Self { raw: Box::new(e) }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.raw.to_string()).into_response()
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

#[derive(Debug)]
pub struct TaskAbortError;

impl std::error::Error for TaskAbortError {}

impl Display for TaskAbortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task was aborted")
    }
}
