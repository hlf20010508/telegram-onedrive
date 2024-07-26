/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use grammers_client::types::{Message, PackedChat};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug)]
pub enum Error {
    DefaultError {
        message: String,
        details: Option<String>,
    },
    HttpHeaderValueError {
        raw: reqwest::header::InvalidHeaderValue,
        message: String,
        details: Option<String>,
    },
    HttpHeaderToStrError {
        raw: reqwest::header::ToStrError,
        message: String,
        details: Option<String>,
    },
    HttpRequestError {
        raw: reqwest::Error,
        message: String,
        details: Option<String>,
    },
    SysIOError {
        raw: std::io::Error,
        message: String,
        details: Option<String>,
    },
    CertGenError {
        raw: rcgen::Error,
        message: String,
        details: Option<String>,
    },
    TlsError {
        raw: native_tls::Error,
        message: String,
        details: Option<String>,
    },
    SocketIOServerBroadcastError {
        raw: socketioxide::BroadcastError,
        message: String,
        details: Option<String>,
    },
    SocketIOClientError {
        raw: rust_socketio::Error,
        message: String,
        details: Option<String>,
    },
    TelegramInvocationError {
        raw: grammers_client::InvocationError,
        message: String,
        details: Option<String>,
    },
    TelegramPackedChatError {
        raw: String,
        message: String,
        details: Option<String>,
    },
    TelegramAuthorizationError {
        raw: grammers_client::client::bots::AuthorizationError,
        message: String,
        details: Option<String>,
    },
    TelegramSignInError {
        raw: grammers_client::SignInError,
        message: String,
        details: Option<String>,
    },
    OneDriveError {
        raw: onedrive_api::Error,
        message: String,
        details: Option<String>,
    },
    SerdeError {
        raw: serde_json::Error,
        message: String,
        details: Option<String>,
    },
    DatabaseError {
        raw: sea_orm::DbErr,
        message: String,
        details: Option<String>,
    },
    ArgError {
        raw: pico_args::Error,
        message: String,
        details: Option<String>,
    },
    ParseIntError {
        raw: std::num::ParseIntError,
        message: String,
        details: Option<String>,
    },
    ParseUrlError {
        raw: url::ParseError,
        message: String,
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

    pub fn respond_error<T>(e: grammers_client::InvocationError, response: T) -> Self
    where
        T: Display,
    {
        Self::new_telegram_invocation(e, "failed to respond message").details(response)
    }

    pub fn trace(self) {
        tracing::debug!("{}", self.to_string());
    }

    pub async fn send(self, message: Arc<Message>) -> Result<Self> {
        message.reply(self.to_string()).await.map_err(|e| {
            Self::new_telegram_invocation(e, "failed to send error message").details(&self)
        })?;

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
            .send_message(chat, self.to_string())
            .await
            .map_err(|e| {
                Error::new_telegram_invocation(e, "failed to send error message").details(&self)
            })?;

        Ok(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_fmt<T>(
            f: &mut std::fmt::Formatter<'_>,
            raw: T,
            message: &String,
            details: &Option<String>,
        ) -> std::fmt::Result
        where
            T: Display,
        {
            match details {
                Some(details) => {
                    write!(f, "{}: {}\ndetails: {}", message, raw, details)
                }
                None => write!(f, "{}: {}", message, raw,),
            }
        }

        match self {
            Self::DefaultError { message, details } => match details {
                Some(details) => {
                    write!(f, "{}\ndetails: {}", message, details)
                }
                None => write!(f, "{}", message),
            },
            Self::HttpHeaderValueError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::HttpHeaderToStrError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::HttpRequestError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::SysIOError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::CertGenError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::TlsError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::SocketIOServerBroadcastError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::SocketIOClientError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::TelegramInvocationError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::TelegramPackedChatError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::TelegramAuthorizationError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::TelegramSignInError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::OneDriveError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::SerdeError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::DatabaseError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::ArgError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::ParseIntError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
            Self::ParseUrlError {
                raw,
                message,
                details,
            } => write_fmt(f, raw, message, details),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
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
