/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::{Client, Config};
use grammers_session::Session;

use crate::env::{Env, TelegramBotEnv};
use crate::error::{Error, Result};

pub struct TelegramBotClient {
    pub client: Client,
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
            ..
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

        let is_authorized = client.is_authorized().await.map_err(|e| {
            Error::context(e, "failed to check telegram bot client authorization state")
        })?;

        if !is_authorized {
            client
                .bot_sign_in(token)
                .await
                .map_err(|e| Error::context(e, "failed to sign in telegram bot"))?;

            client
                .session()
                .save_to_file(session_path)
                .map_err(|e| Error::context(e, "failed to save session for telegram bot client"))?;
        }

        Ok(Self { client })
    }
}
