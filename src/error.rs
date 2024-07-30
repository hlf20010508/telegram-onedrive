/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use grammers_client::types::PackedChat;
use proc_macros::{add_context, add_trace};
use std::fmt::Display;

use crate::client::TelegramClient;
use crate::message::TelegramMessage;

#[derive(Debug)]
pub enum Error {
    Default {
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    HttpHeaderValue {
        raw: reqwest::header::InvalidHeaderValue,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    HttpHeaderToStr {
        raw: reqwest::header::ToStrError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    HttpRequest {
        raw: reqwest::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SysIO {
        raw: std::io::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    CertGen {
        raw: rcgen::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    Tls {
        raw: native_tls::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SocketIOServerBroadcast {
        raw: socketioxide::BroadcastError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    SocketIOClient {
        raw: rust_socketio::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramInvocation {
        raw: grammers_client::InvocationError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramPackedChat {
        raw: String,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramAuthorization {
        raw: grammers_client::client::bots::AuthorizationError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    TelegramSignIn {
        raw: grammers_client::SignInError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    OneDrive {
        raw: onedrive_api::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    Serde {
        raw: serde_json::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    Database {
        raw: sea_orm::DbErr,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    Arg {
        raw: pico_args::Error,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    ParseInt {
        raw: std::num::ParseIntError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    ParseUrl {
        raw: url::ParseError,
        message: String,
        contexts: Vec<String>,
        details: Vec<String>,
    },
    Zip {
        raw: zip::result::ZipError,
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
        Self::Default {
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_http_header_value<T>(e: reqwest::header::InvalidHeaderValue, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpHeaderValue {
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
        Self::HttpHeaderToStr {
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
        Self::HttpRequest {
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
        Self::SysIO {
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
        Self::CertGen {
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
        Self::Tls {
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
        Self::SocketIOServerBroadcast {
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
        Self::SocketIOClient {
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
        Self::TelegramInvocation {
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
        Self::TelegramPackedChat {
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
        Self::TelegramAuthorization {
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
        Self::TelegramSignIn {
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
        Self::OneDrive {
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
        Self::Serde {
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
        Self::Database {
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
        Self::Arg {
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
        Self::ParseInt {
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
        Self::ParseUrl {
            raw: e,
            message: message.to_string(),
            contexts: Vec::new(),
            details: Vec::new(),
        }
    }

    pub fn new_zip<T>(e: zip::result::ZipError, message: T) -> Self
    where
        T: Display,
    {
        Self::Zip {
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
            Self::Default { details, .. }
            | Self::HttpHeaderValue { details, .. }
            | Self::HttpHeaderToStr { details, .. }
            | Self::HttpRequest { details, .. }
            | Self::SysIO { details, .. }
            | Self::CertGen { details, .. }
            | Self::Tls { details, .. }
            | Self::SocketIOServerBroadcast { details, .. }
            | Self::SocketIOClient { details, .. }
            | Self::TelegramInvocation { details, .. }
            | Self::TelegramPackedChat { details, .. }
            | Self::TelegramAuthorization { details, .. }
            | Self::TelegramSignIn { details, .. }
            | Self::OneDrive { details, .. }
            | Self::Serde { details, .. }
            | Self::Database { details, .. }
            | Self::Arg { details, .. }
            | Self::ParseInt { details, .. }
            | Self::ParseUrl { details, .. }
            | Self::Zip { details, .. } => details.push(detail.to_string()),
        }

        self
    }

    pub fn context<T>(mut self, context: T) -> Self
    where
        T: Display,
    {
        match &mut self {
            Self::Default { contexts, .. }
            | Self::HttpHeaderValue { contexts, .. }
            | Self::HttpHeaderToStr { contexts, .. }
            | Self::HttpRequest { contexts, .. }
            | Self::SysIO { contexts, .. }
            | Self::CertGen { contexts, .. }
            | Self::Tls { contexts, .. }
            | Self::SocketIOServerBroadcast { contexts, .. }
            | Self::SocketIOClient { contexts, .. }
            | Self::TelegramInvocation { contexts, .. }
            | Self::TelegramPackedChat { contexts, .. }
            | Self::TelegramAuthorization { contexts, .. }
            | Self::TelegramSignIn { contexts, .. }
            | Self::OneDrive { contexts, .. }
            | Self::Serde { contexts, .. }
            | Self::Database { contexts, .. }
            | Self::Arg { contexts, .. }
            | Self::ParseInt { contexts, .. }
            | Self::ParseUrl { contexts, .. }
            | Self::Zip { contexts, .. } => contexts.insert(0, context.to_string()),
        }

        self
    }

    pub fn trace(self) {
        tracing::error!("{}", self.to_string());
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
            Self::Default {
                message,
                contexts,
                details,
            } => {
                let base = message.clone();
                write_fmt(f, base, contexts, details)
            }
            Self::HttpHeaderValue {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::HttpHeaderToStr {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::HttpRequest {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SysIO {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::CertGen {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::Tls {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SocketIOServerBroadcast {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::SocketIOClient {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramInvocation {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramPackedChat {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramAuthorization {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::TelegramSignIn {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::OneDrive {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::Serde {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::Database {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::Arg {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::ParseInt {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::ParseUrl {
                raw,
                message,
                contexts,
                details,
            } => {
                let base = format!("{}: {}", message, raw);
                write_fmt(f, base, contexts, details)
            }
            Self::Zip {
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
    fn details<U>(self, message_details: U) -> Self
    where
        U: Display,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.details(message_details)),
        }
    }

    fn context<U>(self, message_context: U) -> Self
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
            Ok(e) | Err(e) => e,
        }
    }
}
