/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;

pub const PATTERN: &str = "/start";

pub async fn handler(message: TelegramMessage, _state: AppState) -> Result<()> {
    message.respond(docs::GREETING).await?;

    Ok(())
}
