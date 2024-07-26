/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;

use crate::client::{TelegramClient, TelegramMessage};
use crate::env::LOG_PATH;
use crate::error::{Result, ResultExt};

pub async fn send_log_file(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    let file = telegram_bot.upload_file(LOG_PATH).await?;

    message
        .respond(InputMessage::default().file(file))
        .await
        .context("log file")?;

    Ok(())
}
