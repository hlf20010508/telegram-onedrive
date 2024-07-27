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
        context: Option<String>,
        details: Option<String>,
    },
    HttpHeaderValueError {
        raw: reqwest::header::InvalidHeaderValue,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    HttpHeaderToStrError {
        raw: reqwest::header::ToStrError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    HttpRequestError {
        raw: reqwest::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    SysIOError {
        raw: std::io::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    CertGenError {
        raw: rcgen::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    TlsError {
        raw: native_tls::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    SocketIOServerBroadcastError {
        raw: socketioxide::BroadcastError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    SocketIOClientError {
        raw: rust_socketio::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    TelegramInvocationError {
        raw: grammers_client::InvocationError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    TelegramPackedChatError {
        raw: String,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    TelegramAuthorizationError {
        raw: grammers_client::client::bots::AuthorizationError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    TelegramSignInError {
        raw: grammers_client::SignInError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    OneDriveError {
        raw: onedrive_api::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    SerdeError {
        raw: serde_json::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    DatabaseError {
        raw: sea_orm::DbErr,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    ArgError {
        raw: pico_args::Error,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    ParseIntError {
        raw: std::num::ParseIntError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
    ParseUrlError {
        raw: url::ParseError,
        message: String,
        context: Option<String>,
        details: Option<String>,
    },
}

impl Error {
    pub fn new<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::DefaultError {
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_http_header_value<T>(e: reqwest::header::InvalidHeaderValue, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpHeaderValueError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_http_header_to_str<T>(e: reqwest::header::ToStrError, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpHeaderToStrError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_http_request<T>(e: reqwest::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::HttpRequestError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_sys_io<T>(e: std::io::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::SysIOError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_cert_gen<T>(e: rcgen::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::CertGenError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_tls<T>(e: native_tls::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::TlsError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_socket_io_server_broadcast<T>(e: socketioxide::BroadcastError, message: T) -> Self
    where
        T: Display,
    {
        Self::SocketIOServerBroadcastError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_socket_io_client<T>(e: rust_socketio::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::SocketIOClientError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_telegram_invocation<T>(e: grammers_client::InvocationError, message: T) -> Self
    where
        T: Display,
    {
        Self::TelegramInvocationError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
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
            context: None,
            details: None,
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
            context: None,
            details: None,
        }
    }

    pub fn new_telegram_sign_in<T>(e: grammers_client::SignInError, message: T) -> Self
    where
        T: Display,
    {
        Self::TelegramSignInError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_onedrive<T>(e: onedrive_api::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::OneDriveError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_serde<T>(e: serde_json::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::SerdeError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_database<T>(e: sea_orm::DbErr, message: T) -> Self
    where
        T: Display,
    {
        Self::DatabaseError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_arg<T>(e: pico_args::Error, message: T) -> Self
    where
        T: Display,
    {
        Self::ArgError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_parse_int<T>(e: std::num::ParseIntError, message: T) -> Self
    where
        T: Display,
    {
        Self::ParseIntError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn new_parse_url<T>(e: url::ParseError, message: T) -> Self
    where
        T: Display,
    {
        Self::ParseUrlError {
            raw: e,
            message: message.to_string(),
            context: None,
            details: None,
        }
    }

    pub fn details<T>(mut self, message_details: T) -> Self
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
            | Self::ParseUrlError { details, .. } => *details = Some(message_details.to_string()),
        }

        self
    }

    pub fn context<T>(mut self, message_context: T) -> Self
    where
        T: Display,
    {
        match &mut self {
            Self::DefaultError { context, .. }
            | Self::HttpHeaderValueError { context, .. }
            | Self::HttpHeaderToStrError { context, .. }
            | Self::HttpRequestError { context, .. }
            | Self::SysIOError { context, .. }
            | Self::CertGenError { context, .. }
            | Self::TlsError { context, .. }
            | Self::SocketIOServerBroadcastError { context, .. }
            | Self::SocketIOClientError { context, .. }
            | Self::TelegramInvocationError { context, .. }
            | Self::TelegramPackedChatError { context, .. }
            | Self::TelegramAuthorizationError { context, .. }
            | Self::TelegramSignInError { context, .. }
            | Self::OneDriveError { context, .. }
            | Self::SerdeError { context, .. }
            | Self::DatabaseError { context, .. }
            | Self::ArgError { context, .. }
            | Self::ParseIntError { context, .. }
            | Self::ParseUrlError { context, .. } => *context = Some(message_context.to_string()),
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
        fn write_fmt<T>(
            f: &mut std::fmt::Formatter<'_>,
            raw: T,
            message: &String,
            context: &Option<String>,
            details: &Option<String>,
        ) -> std::fmt::Result
        where
            T: Display,
        {
            let mut output = format!("{}: {}", message, raw);

            if let Some(context) = context {
                output.push_str(&format!("\ncontext: {}", context));
            }

            if let Some(details) = details {
                output.push_str(&format!("\ndetails: {}", details));
            }

            write!(f, "{}", output)
        }

        match self {
            Self::DefaultError {
                message,
                context,
                details,
            } => {
                let mut output = message.clone();

                if let Some(context) = context {
                    output.push_str(&format!("\ncontext: {}", context));
                }

                if let Some(details) = details {
                    output.push_str(&format!("\ndetails: {}", details));
                }

                write!(f, "{}", output)
            }
            Self::HttpHeaderValueError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::HttpHeaderToStrError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::HttpRequestError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::SysIOError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::CertGenError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::TlsError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::SocketIOServerBroadcastError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::SocketIOClientError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::TelegramInvocationError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::TelegramPackedChatError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::TelegramAuthorizationError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::TelegramSignInError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::OneDriveError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::SerdeError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::DatabaseError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::ArgError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::ParseIntError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
            Self::ParseUrlError {
                raw,
                message,
                context,
                details,
            } => write_fmt(f, raw, message, context, details),
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
