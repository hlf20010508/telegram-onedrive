/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use futures::FutureExt;
use grammers_client::types::Message;
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use rust_socketio::asynchronous::{
    Client as SocketIoClient, ClientBuilder as SocketIoClientBuilder,
};
use rust_socketio::Payload;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::auth_server::{SERVER_PORT, TG_CODE_EVENT};
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
            server_url,
            ..
        }: &Env,
    ) -> Result<()> {
        if !self.is_authorized().await? {
            let token = self
                .client
                .request_login_code(phone_number)
                .await
                .map_err(|e| Error::context(e, "failed to request telegram user login code"))?;

            message
                .respond(format!(
                    "Please visit {} to input your code to login to Telegram.",
                    server_url
                ))
                .await
                .map_err(|e| Error::context(e, "failed to respond telegram code server url"))?;

            let (socketio_client, mut rx) = Self::socketio_client().await?;

            loop {
                let code = rx
                    .recv()
                    .await
                    .ok_or_else(|| Error::new("failed to receive code"))?;

                message
                    .respond("Code received, logining...")
                    .await
                    .map_err(|e| Error::context(e, "failed to respond telegram code received"))?;

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

            socketio_client
                .disconnect()
                .await
                .map_err(|e| Error::context(e, "failed to disconnect from auth server"))?;

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

    async fn socketio_client() -> Result<(SocketIoClient, mpsc::Receiver<String>)> {
        let (tx, rx) = mpsc::channel(1);

        let socketio_client =
            SocketIoClientBuilder::new(format!("http://127.0.0.1:{}/", SERVER_PORT))
                .on(TG_CODE_EVENT, move |payload, _socket| {
                    let tx = tx.clone();
                    async move {
                        if let Payload::Text(values) = payload {
                            if let Some(value) = values.get(0) {
                                let code =
                                    serde_json::from_value::<String>(value.to_owned()).unwrap();

                                tx.send(code).await.unwrap();
                            }
                        }
                    }
                    .boxed()
                })
                .connect()
                .await
                .map_err(|e| Error::context(e, "failed to connect to auth server"))?;

        Ok((socketio_client, rx))
    }
}
