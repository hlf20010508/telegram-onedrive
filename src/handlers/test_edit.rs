/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace, check_in_group, check_senders};

use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;

pub const PATTERN: &str = "/testEdit";

#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let response = "test response";
    let respond_message = message.respond(response).await.details(response)?;

    let telegram_bot = &state.telegram_bot;

    for i in 0..10 {
        telegram_bot
            .edit_message(
                message.chat(),
                respond_message.id(),
                format!("test edit {}", i),
            )
            .await?;
    }

    Ok(())
}
