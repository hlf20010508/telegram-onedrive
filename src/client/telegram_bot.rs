/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::env::{Env, TelegramBotEnv};
use crate::error::{Error, Result};
use grammers_client::types::User;
use grammers_client::{Client, Config};
use grammers_session::Session;

pub struct TelegramBotClient {
    pub client: Client,
    pub user: User,
}

impl TelegramBotClient {
    pub async fn new(
        Env {
            telegram_bot:
                TelegramBotEnv {
                    api_id,
                    api_hash,
                    token,
                    session_path,
                    params,
                },
        }: &Env,
    ) -> Result<Self> {
        let session = Session::load_file_or_create(session_path).map_err(|e| {
            Error::context(
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

        let client = Client::connect(config)
            .await
            .map_err(|e| Error::context(e, "failed to create telegram bot client"))?;

        let user = if !client.is_authorized().await.map_err(|e| {
            Error::context(e, "failed to check telegram bot client authorization state")
        })? {
            let user = client
                .bot_sign_in(token)
                .await
                .map_err(|e| Error::context(e, "failed to sign in telegram bot account"))?;

            client
                .session()
                .save_to_file(session_path)
                .map_err(|e| Error::context(e, "failed to save session for telegram bot client"))?;

            user
        } else {
            client
                .get_me()
                .await
                .map_err(|e| Error::context(e, "failed to get telegram bot client user"))?
        };

        Ok(Self { client, user })
    }
}
