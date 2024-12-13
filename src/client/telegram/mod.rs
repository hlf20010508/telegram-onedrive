/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod file;
mod message;

use crate::{
    env::{Env, TelegramBotEnv, TelegramUserEnv, ENV},
    message::TelegramMessage,
};
use anyhow::{anyhow, Context, Result};
use grammers_client::{session::Session, Client, Config, SignInError};
use message::ChatMessageVecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, Mutex};

// messages to be sent or edited in each chat
type ChatMessageQueue = Arc<Mutex<ChatMessageVecDeque>>;

#[derive(Clone)]
pub enum TelegramClient {
    Bot {
        client: Client,
        chat_message_queue: ChatMessageQueue,
    },
    User {
        client: Client,
        chat_message_queue: ChatMessageQueue,
    },
}

impl TelegramClient {
    pub async fn new_bot() -> Result<Self> {
        let Env {
            telegram_bot:
                TelegramBotEnv {
                    api_id,
                    api_hash,
                    token,
                    session_path,
                    params,
                },
            ..
        } = ENV.get().unwrap();

        let session = Session::load_file_or_create(session_path)
            .context("failed to load or create session for telegram bot client")?;

        let config = Config {
            session,
            api_id: *api_id,
            api_hash: api_hash.clone(),
            params: params.clone(),
        };

        let client = Client::connect(config)
            .await
            .context("failed to create telegram bot client")?;

        let is_authorized = client
            .is_authorized()
            .await
            .context("failed to check telegram bot client authorization state")?;

        if !is_authorized {
            client
                .bot_sign_in(token)
                .await
                .context("failed to sign in telegram bot")?;

            client
                .session()
                .save_to_file(session_path)
                .context("failed to save session for telegram bot client")?;
        }

        let telegram_client = Self::Bot {
            client,
            chat_message_queue: Arc::new(Mutex::new(ChatMessageVecDeque::new())),
        };

        telegram_client.run_message_loop();

        Ok(telegram_client)
    }

    pub async fn new_user() -> Result<Self> {
        let Env {
            telegram_user:
                TelegramUserEnv {
                    api_id,
                    api_hash,
                    session_path,
                    params,
                    ..
                },
            ..
        } = ENV.get().unwrap();

        let session = Session::load_file_or_create(session_path)
            .context("failed to load or create session for telegram user client")?;

        let config = Config {
            session,
            api_id: *api_id,
            api_hash: api_hash.clone(),
            params: params.clone(),
        };

        let client = Client::connect(config)
            .await
            .context("failed to create telegram user client")?;

        let telegram_client = Self::User {
            client,
            chat_message_queue: Arc::new(Mutex::new(ChatMessageVecDeque::new())),
        };

        telegram_client.run_message_loop();

        Ok(telegram_client)
    }

    pub const fn raw(&self) -> &Client {
        match self {
            Self::Bot { client, .. } | Self::User { client, .. } => client,
        }
    }

    fn chat_message_queue(&self) -> ChatMessageQueue {
        match self {
            Self::Bot {
                chat_message_queue, ..
            }
            | Self::User {
                chat_message_queue, ..
            } => chat_message_queue.clone(),
        }
    }

    pub async fn login(&self, message: TelegramMessage, mut rx: Receiver<String>) -> Result<()> {
        if !self.is_authorized().await? {
            let Env {
                telegram_user:
                    TelegramUserEnv {
                        phone_number,
                        password,
                        session_path,
                        ..
                    },
                server_uri,
                ..
            } = ENV.get().unwrap();

            let client = self.raw();

            let response = "Sending telegram login code...\nThis may take a while.";
            message.respond(response).await.context(response)?;

            let token = client
                .request_login_code(phone_number)
                .await
                .context("failed to request telegram user login code")?;

            let response = format!(
                "Please visit {} to input your code to login to Telegram.",
                server_uri
            );
            message.respond(response.as_str()).await.context(response)?;

            loop {
                let code = rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow!("failed to receive telegram code"))?;

                let response = "Code received, logining...";
                message.respond(response).await.context(response)?;

                match client.sign_in(&token, &code).await {
                    Ok(_) => {}
                    Err(SignInError::PasswordRequired(password_token)) => match password {
                        Some(password) => {
                            client
                                .check_password(password_token, password)
                                .await
                                .context("failed to pass telegram user 2FA")?;

                            break;
                        }
                        None => Err(anyhow!("password for telegram user 2FA required"))?,
                    },
                    Err(SignInError::InvalidCode) => {
                        message.respond("Code invalid, please input again.").await?;
                    }
                    Err(e) => Err(e).context("failed to sign in telegram user")?,
                };
            }

            client
                .session()
                .save_to_file(session_path)
                .context("failed to save session for telegram user client")?;
        }

        Ok(())
    }

    pub async fn is_authorized(&self) -> Result<bool> {
        self.raw()
            .is_authorized()
            .await
            .context("failed to check telegram user client authorization state")
    }
}
