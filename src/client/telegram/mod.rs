/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod file;
mod message;

use grammers_client::session::Session;
use grammers_client::{Client, Config, SignInError};

pub use message::TelegramMessage;

use super::utils::{socketio_client, socketio_disconnect};
use crate::auth_server::TG_CODE_EVENT;
use crate::env::{Env, TelegramBotEnv, TelegramUserEnv};
use crate::error::{Error, Result, ResultExt};

#[derive(Clone)]
pub enum TelegramClient {
    Bot { client: Client },
    User { client: Client },
}

impl TelegramClient {
    pub async fn new_bot(
        Env {
            telegram_bot:
                TelegramBotEnv {
                    api_id,
                    api_hash,
                    token,
                    session_path,
                    params,
                },
            ..
        }: &Env,
    ) -> Result<Self> {
        let session = Session::load_file_or_create(session_path).map_err(|e| {
            Error::new_sys_io(
                e,
                "failed to load or create session for telegram bot client",
            )
        })?;

        let config = Config {
            session,
            api_id: *api_id,
            api_hash: api_hash.clone(),
            params: params.clone(),
        };

        let client = Client::connect(config).await.map_err(|e| {
            Error::new_telegram_authorization(e, "failed to create telegram bot client")
        })?;

        let is_authorized = client.is_authorized().await.map_err(|e| {
            Error::new_telegram_invocation(
                e,
                "failed to check telegram bot client authorization state",
            )
        })?;

        if !is_authorized {
            client.bot_sign_in(token).await.map_err(|e| {
                Error::new_telegram_authorization(e, "failed to sign in telegram bot")
            })?;

            client.session().save_to_file(session_path).map_err(|e| {
                Error::new_sys_io(e, "failed to save session for telegram bot client")
            })?;
        }

        Ok(Self::Bot { client })
    }

    pub async fn new_user(
        Env {
            telegram_user:
                TelegramUserEnv {
                    api_id,
                    api_hash,
                    session_path,
                    params,
                    ..
                },
            ..
        }: &Env,
    ) -> Result<Self> {
        let session = Session::load_file_or_create(session_path).map_err(|e| {
            Error::new_sys_io(
                e,
                "failed to load or create session for telegram user client",
            )
        })?;

        let config = Config {
            session,
            api_id: *api_id,
            api_hash: api_hash.clone(),
            params: params.clone(),
        };

        let client = Client::connect(config).await.map_err(|e| {
            Error::new_telegram_authorization(e, "failed to create telegram user client")
        })?;

        Ok(Self::User { client })
    }

    fn client(&self) -> &Client {
        match self {
            Self::Bot { client } | Self::User { client } => client,
        }
    }

    pub async fn login(
        &self,
        message: TelegramMessage,
        Env {
            telegram_user:
                TelegramUserEnv {
                    phone_number,
                    password,
                    session_path,
                    ..
                },
            port,
            server_uri,
            use_reverse_proxy,
            ..
        }: &Env,
    ) -> Result<()> {
        let client = self.client();

        let response = "Logining into Telegram...";
        message.respond(response).await.details(response)?;

        if !self.is_authorized().await? {
            let token = client.request_login_code(phone_number).await.map_err(|e| {
                Error::new_telegram_authorization(e, "failed to request telegram user login code")
            })?;

            let response = format!(
                "Please visit {} to input your code to login to Telegram.",
                server_uri
            );
            message.respond(response.as_str()).await.details(response)?;

            let (socketio_client, mut rx) =
                socketio_client(TG_CODE_EVENT, port.to_owned(), use_reverse_proxy.to_owned())
                    .await?;

            loop {
                let code = rx
                    .recv()
                    .await
                    .ok_or_else(|| Error::new("failed to receive telegram code"))?;

                let response = "Code received, logining...";
                message.respond(response).await.details(response)?;

                match client.sign_in(&token, &code).await {
                    Ok(_) => {}
                    Err(SignInError::PasswordRequired(password_token)) => match password {
                        Some(password) => {
                            client
                                .check_password(password_token, password)
                                .await
                                .map_err(|e| {
                                    Error::new_telegram_sign_in(
                                        e,
                                        "failed to pass telegram user 2FA",
                                    )
                                })?;

                            break;
                        }
                        None => Err(Error::new("password for telegram user 2FA required"))?,
                    },
                    Err(SignInError::InvalidCode) => {
                        message
                            .respond("Code invalid, please input again.")
                            .await
                            .context("telegram sign in code invalid")?;
                    }
                    Err(e) => Err(Error::new_telegram_sign_in(
                        e,
                        "failed to sign in telegram user",
                    ))?,
                };
            }

            socketio_disconnect(socketio_client).await?;

            client.session().save_to_file(session_path).map_err(|e| {
                Error::new_sys_io(e, "failed to save session for telegram user client")
            })?;
        }

        Ok(())
    }

    pub async fn is_authorized(&self) -> Result<bool> {
        self.client().is_authorized().await.map_err(|e| {
            Error::new_telegram_invocation(
                e,
                "failed to check telegram user client authorization state",
            )
        })
    }
}
