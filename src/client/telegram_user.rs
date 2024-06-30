/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use std::sync::Arc;

use super::utils::{socketio_client, socketio_disconnect};
use crate::auth_server::TG_CODE_EVENT;
use crate::env::{Env, TelegramUserEnv};
use crate::error::{Error, Result};

pub struct TelegramUserClient {
    pub client: Client,
}

impl TelegramUserClient {
    pub async fn new(
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
            Error::context(
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

        let client = Client::connect(config)
            .await
            .map_err(|e| Error::context(e, "failed to create telegram user client"))?;

        Ok(Self { client })
    }

    pub async fn login(
        &self,
        message: Arc<Message>,
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
        let response = "Logining into Telegram...";
        message
            .respond(response)
            .await
            .map_err(|e| Error::respond_error(e, response))?;

        if !self.is_authorized().await? {
            let token = self
                .client
                .request_login_code(phone_number)
                .await
                .map_err(|e| Error::context(e, "failed to request telegram user login code"))?;

            let response = format!(
                "Please visit {} to input your code to login to Telegram.",
                server_uri
            );
            message
                .respond(response.as_str())
                .await
                .map_err(|e| Error::respond_error(e, response))?;

            let (socketio_client, mut rx) =
                socketio_client(TG_CODE_EVENT, port.to_owned(), use_reverse_proxy.to_owned())
                    .await?;

            loop {
                let code = rx
                    .recv()
                    .await
                    .ok_or_else(|| Error::new("failed to receive telegram code"))?;

                let response = "Code received, logining...";
                message
                    .respond(response)
                    .await
                    .map_err(|e| Error::respond_error(e, response))?;

                match self.client.sign_in(&token, &code).await {
                    Ok(_) => {}
                    Err(SignInError::PasswordRequired(password_token)) => match password {
                        Some(password) => {
                            self.client
                                .check_password(password_token, password)
                                .await
                                .map_err(|e| {
                                    Error::context(e, "failed to pass telegram user 2FA")
                                })?;

                            break;
                        }
                        None => Err(Error::new("password for telegram user 2FA required"))?,
                    },
                    Err(SignInError::InvalidCode) => {
                        message
                            .respond("Code invalid, please input again.")
                            .await
                            .map_err(|e| Error::context(e, "failed to respond code invalid"))?;
                    }
                    Err(e) => Err(Error::context(e, "failed to sign in telegram user"))?,
                };
            }

            socketio_disconnect(socketio_client).await?;

            self.client
                .session()
                .save_to_file(session_path)
                .map_err(|e| {
                    Error::context(e, "failed to save session for telegram user client")
                })?;
        }

        Ok(())
    }

    async fn is_authorized(&self) -> Result<bool> {
        self.client.is_authorized().await.map_err(|e| {
            Error::context(
                e,
                "failed to check telegram user client authorization state",
            )
        })
    }
}
