/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace};

use crate::client::TelegramClient;
use crate::env::LOGS_PATH;
use crate::error::Result;
use crate::message::TelegramMessage;

#[add_context]
#[add_trace]
pub async fn send_log_file(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    let file = telegram_bot.upload_file(LOGS_PATH).await?;

    message.respond(InputMessage::default().file(file)).await?;

    Ok(())
}
