/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use grammers_client::types::PackedChat;
use std::fmt::Display;

use crate::client::TelegramClient;
use crate::message::TelegramMessage;

#[derive(Debug)]
pub enum Error {
    DefaultError {
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    HttpHeaderValueError {
        raw: reqwest::header::InvalidHeaderValue,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    HttpHeaderToStrError {
        raw: reqwest::header::ToStrError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    HttpRequestError {
        raw: reqwest::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SysIOError {
        raw: std::io::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    CertGenError {
        raw: rcgen::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TlsError {
        raw: native_tls::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SocketIOServerBroadcastError {
        raw: socketioxide::BroadcastError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SocketIOClientError {
        raw: rust_socketio::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramInvocationError {
        raw: grammers_client::InvocationError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramPackedChatError {
        raw: String,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramAuthorizationError {
        raw: grammers_client::client::bots::AuthorizationError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramSignInError {
        raw: grammers_client::SignInError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    OneDriveError {
        raw: onedrive_api::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SerdeError {
        raw: serde_json::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    DatabaseError {
        raw: sea_orm::DbErr,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    ArgError {
        raw: pico_args::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    ParseIntError {
        raw: std::num::ParseIntError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    ParseUrlError {
        raw: url::ParseError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
}

impl Error {
    pub fn new<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::DefaultError {
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_http_header_value<T>(e: reqwest::header::InvalidHeaderValue, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpHeaderValueError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_http_header_to_str<T>(e: reqwest::header::ToStrError, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpHeaderToStrError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_http_request<T>(e: reqwest::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpRequestError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_sys_io<T>(e: std::io::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::SysIOError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_cert_gen<T>(e: rcgen::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::CertGenError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_tls<T>(e: native_tls::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::TlsError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_socket_io_server_broadcast<T>(e: socketioxide::BroadcastError, message: T) -> Self
    where
        T: Display,
    {
        Self::SocketIOServerBroadcastError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_socket_io_client<T>(e: rust_socketio::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::SocketIOClientError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_telegram_invocation<T>(e: grammers_client::InvocationError, message: T) -> Self
    where
        T: Display,
    {
        Self::TelegramInvocationError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_telegram_packed_chat<T, U>(e: T, message: U) -> Self
    where
        T: Display,
        U: Display,
    {
        Self::TelegramPackedChatError {
            raw: e.to_string(),
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_telegram_authorization<T>(
        e: grammers_client::client::bots::AuthorizationError,
        message: T,
    ) -> Self
    where
        T: Display,
    {
        Self::TelegramAuthorizationError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_telegram_sign_in<T>(e: grammers_client::SignInError, message: T) -> Self
    where
        T: Display,
    {
        Self::TelegramSignInError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_onedrive<T>(e: onedrive_api::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::OneDriveError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_serde<T>(e: serde_json::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::SerdeError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_database<T>(e: sea_orm::DbErr, message: T) -> Self
    where
        T: Display,
    {
        Self::DatabaseError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_arg<T>(e: pico_args::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::ArgError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_parse_int<T>(e: std::num::ParseIntError, message: T) -> Self
    where
        T: Display,
    {
        Self::ParseIntError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_parse_url<T>(e: url::ParseError, message: T) -> Self
    where
        T: Display,
    {
        Self::ParseUrlError {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn details<T>(mut self, detail: T) -> Self
    where
        T: Display,
    {
        match &mut self {
            Self::DefaultError { details, .. }
            | Self::HttpHeaderValueError { details, .. }
            | Self::HttpHeaderToStrError { details, .. }
            | Self::HttpRequestError { details, .. }
            | Self::SysIOError { details, .. }
            | Self::CertGenError { details, .. }
            | Self::TlsError { details, .. }
            | Self::SocketIOServerBroadcastError { details, .. }
            | Self::SocketIOClientError { details, .. }
            | Self::TelegramInvocationError { details, .. }
            | Self::TelegramPackedChatError { details, .. }
            | Self::TelegramAuthorizationError { details, .. }
            | Self::TelegramSignInError { details, .. }
            | Self::OneDriveError { details, .. }
            | Self::SerdeError { details, .. }
            | Self::DatabaseError { details, .. }
            | Self::ArgError { details, .. }
            | Self::ParseIntError { details, .. }
            | Self::ParseUrlError { details, .. } => details.push(detail.to_string()),
        }

        self
    }

    pub fn context<T>(mut self, context: T) -> Self
    where
        T: Display,
    {
        match &mut self {
            Self::DefaultError { contexts, .. }
            | Self::HttpHeaderValueError { contexts, .. }
            | Self::HttpHeaderToStrError { contexts, .. }
            | Self::HttpRequestError { contexts, .. }
            | Self::SysIOError { contexts, .. }
            | Self::CertGenError { contexts, .. }
            | Self::TlsError { contexts, .. }
            | Self::SocketIOServerBroadcastError { contexts, .. }
            | Self::SocketIOClientError { contexts, .. }
            | Self::TelegramInvocationError { contexts, .. }
            | Self::TelegramPackedChatError { contexts, .. }
            | Self::TelegramAuthorizationError { contexts, .. }
            | Self::TelegramSignInError { contexts, .. }
            | Self::OneDriveError { contexts, .. }
            | Self::SerdeError { contexts, .. }
            | Self::DatabaseError { contexts, .. }
            | Self::ArgError { contexts, .. }
            | Self::ParseIntError { contexts, .. }
            | Self::ParseUrlError { contexts, .. } => contexts.insert(0, context.to_string()),
        }

        self
    }

    pub fn trace(self) {
        tracing::debug!("{}", self.to_string());
    }

    pub async fn send(self, message: TelegramMessage) -> Result<Self> {
        message
            .reply(self.to_string())
            .await
            .context("error message")
            .details(&self)?;

        Ok(self)
    }

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
            contexts: &Vec<String>,
            details: &Vec<String>,
        ) -> std::fmt::Result {
            if !contexts.is_empty() {
                let contexts = contexts.join("\n-");
                base.push_str(&format!("\ncontexts:\n-{}", contexts));
            }

            if !details.is_empty() {
                let details = details.join("\n--------\n");
                base.push_str(&format!("\ndetails:\n{}", details));
            }

            write!(f, "{}", base)
        }

        match self {
            Self::DefaultError {
                message,
                contexts,
                details,
            } => {
                let base = message.clone();
                write_fmt(f, base, contexts, details)
            }
            Self::HttpHeaderValueError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::HttpHeaderToStrError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::HttpRequestError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SysIOError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::CertGenError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TlsError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SocketIOServerBroadcastError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SocketIOClientError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramInvocationError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramPackedChatError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramAuthorizationError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramSignInError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::OneDriveError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SerdeError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::DatabaseError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::ArgError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::ParseIntError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::ParseUrlError {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
        }
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
}

impl<T> ResultExt<T> for Result<T> {
    fn details<U>(self, message_details: U) -> Result<T>
    where
        U: Display,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.details(message_details)),
        }
    }

    fn context<U>(self, message_context: U) -> Result<T>
    where
        U: Display,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.context(message_context)),
        }
    }
}

pub trait ResultUnwrapExt {
    fn unwrap_both(self) -> Error;
}

impl ResultUnwrapExt for Result<Error> {
    fn unwrap_both(self) -> Error {
        match self {
            Ok(e) => e,
            Err(e) => e,
        }
    }
}
