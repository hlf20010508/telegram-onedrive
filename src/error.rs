/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::fmt::Display;

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

    pub fn trace(self) {
        tracing::debug!("{}", self.0);
    }

    pub async fn send(self, message: Message) -> Result<Self> {
        message
            .reply(self.0.clone())
            .await
            .map_err(|e| Error::details(e, "failed to send error message", self.0.clone()))?;

        Ok(self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
