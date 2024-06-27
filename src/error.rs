/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::fmt::Display;

use grammers_client::types::Chat;

use crate::client::telegram_bot::TelegramBotClient;

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

    // pub async fn send(self, telegram_bot: &TelegramBotClient, chat: Chat) -> Self {
    //     telegram_bot
    //         .client
    //         .send_message(chat, self.0.clone())
    //         .await
    //         .unwrap();

    //     self
    // }
}

pub type Result<T> = std::result::Result<T, Error>;
