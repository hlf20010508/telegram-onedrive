/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

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
}

pub type Result<T> = std::result::Result<T, Error>;
