/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use grammers_client::InputMessage;

use crate::client::TelegramMessage;
use crate::error::{Result, ResultExt};
use crate::state::AppState;

pub const PATTERN: &str = "/help";

pub async fn handler(message: TelegramMessage, _state: AppState) -> Result<()> {
    message
        .respond(InputMessage::html(docs::GREETING))
        .await
        .context("help docs")?;

    Ok(())
}
