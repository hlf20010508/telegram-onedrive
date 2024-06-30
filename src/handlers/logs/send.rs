/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use grammers_client::InputMessage;
use std::sync::Arc;

use crate::client::TelegramBotClient;
use crate::env::LOG_PATH;
use crate::error::{Error, Result};

pub async fn send_log_file(telegram_bot: &TelegramBotClient, message: Arc<Message>) -> Result<()> {
    let file = telegram_bot
        .client
        .upload_file(LOG_PATH)
        .await
        .map_err(|e| Error::context(e, "failed to upload log file"))?;

    message
        .respond(InputMessage::default().file(file))
        .await
        .map_err(|e| Error::context(e, "failed to respond log file"))?;

    Ok(())
}
