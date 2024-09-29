/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use crate::{error::Result, message::TelegramMessage, state::AppState};
use proc_macros::{add_context, add_trace};

pub const PATTERN: &str = "/start";

#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, _state: AppState) -> Result<()> {
    message.respond(docs::GREETING).await?;

    Ok(())
}
