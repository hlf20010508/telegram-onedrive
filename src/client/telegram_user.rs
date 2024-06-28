/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::env::{Env, TelegramUserEnv};
use crate::error::{Error, Result};
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;

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
        Env {
            telegram_user:
                TelegramUserEnv {
                    phone_number,
                    password,
                    session_path,
                    ..
                },
            ..
        }: &Env,
    ) -> Result<()> {
        if !self.is_authorized().await? {
            let token = self
                .client
                .request_login_code(phone_number)
                .await
                .map_err(|e| Error::context(e, "failed to request telegram user login code"))?;

            let code = self.get_code().await;

            match self.client.sign_in(&token, &code).await {
                Ok(user) => user,
                Err(SignInError::PasswordRequired(password_token)) => match password {
                    Some(password) => self
                        .client
                        .check_password(password_token, password)
                        .await
                        .map_err(|e| Error::context(e, "failed to pass telegram user 2FA"))?,
                    None => Err(Error::new("password for telegram user 2FA required"))?,
                },
                Err(e) => Err(Error::context(e, "failed to sign in telegram user"))?,
            };

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

    async fn get_code(&self) -> String {
        todo!();
    }
}
