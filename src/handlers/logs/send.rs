/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;

use crate::client::TelegramClient;
use crate::env::LOG_PATH;
use crate::error::Result;
use crate::message::TelegramMessage;

pub async fn send_log_file(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    let file = telegram_bot.upload_file(LOG_PATH).await?;

    message.respond(InputMessage::default().file(file)).await?;

    Ok(())
}
