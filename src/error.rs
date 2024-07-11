/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use grammers_client::types::Message;
use grammers_session::PackedChat;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug)]
pub struct Error(pub String);

impl Error {
    pub fn new<T>(message: T) -> Self
    where
        T: Display,
    {
        Self(message.to_string())
    }

    pub fn context<T, U>(e: T, message: U) -> Self
    where
        T: Display,
        U: Display,
    {
        Self(format!("{}: {}", message, e))
    }

    pub fn details<T, U, V>(e: T, message: U, details: V) -> Self
    where
        T: Display,
        U: Display,
        V: Display,
    {
        Self(format!("{}: {}\ndetails:{}", message, e, details))
    }

    pub fn respond_error<T, U>(e: T, response: U) -> Self
    where
        T: Display,
        U: Display,
    {
        Self::details(e, "failed to respond message", response)
    }

    pub fn trace(self) {
        tracing::debug!("{}", self.0);
    }

    pub async fn send(self, message: Arc<Message>) -> Result<Self> {
        message
            .reply(self.0.clone())
            .await
            .map_err(|e| Error::details(e, "failed to send error message", self.0.clone()))?;

        Ok(self)
    }

    pub async fn send_chat<C>(
        self,
        telegram_client: &grammers_client::Client,
        chat: C,
    ) -> Result<Self>
    where
        C: Into<PackedChat>,
    {
        telegram_client
            .send_message(chat, self.0.clone())
            .await
            .map_err(|e| Error::details(e, "failed to send error message", self.0.clone()))?;

        Ok(self)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultExt {
    fn unwrap_both(self) -> Error;
}

impl ResultExt for Result<Error> {
    fn unwrap_both(self) -> Error {
        match self {
            Ok(e) => e,
            Err(e) => e,
        }
    }
}
