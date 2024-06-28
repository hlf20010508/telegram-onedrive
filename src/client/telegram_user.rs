/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::env::{Env, TelegramUserEnv};
use crate::error::{Error, Result};
use grammers_client::types::User;
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;

pub struct TelegramUserClient {
    pub client: Client,
    pub user: User,
}

impl TelegramUserClient {
    pub async fn new(
        Env {
            telegram_user:
                TelegramUserEnv {
                    api_id,
                    api_hash,
                    phone_number,
                    password,
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

        let user = if !client.is_authorized().await.map_err(|e| {
            Error::context(
                e,
                "failed to check telegram user client authorization state",
            )
        })? {
            let token = client
                .request_login_code(phone_number)
                .await
                .map_err(|e| Error::context(e, "failed to request telegram user login code"))?;

            let code = get_code().await;

            let user = match client.sign_in(&token, &code).await {
                Ok(user) => user,
                Err(SignInError::PasswordRequired(password_token)) => match password {
                    Some(password) => client
                        .check_password(password_token, password)
                        .await
                        .map_err(|e| Error::context(e, "failed to pass telegram user 2FA"))?,
                    None => Err(Error::new("password for telegram user 2FA required"))?,
                },
                Err(e) => Err(Error::context(e, "failed to sign in telegram user"))?,
            };

            client.session().save_to_file(session_path).map_err(|e| {
                Error::context(e, "failed to save session for telegram user client")
            })?;

            user
        } else {
            client
                .get_me()
                .await
                .map_err(|e| Error::context(e, "failed to get telegram user client user"))?
        };

        Ok(Self { client, user })
    }
}

async fn get_code() -> String {
    todo!();
}
