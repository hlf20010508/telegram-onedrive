/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace};

use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;

#[add_context]
#[add_trace]
pub async fn login_to_telegram(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let env = &state.env;

    telegram_user.login(message.clone(), env).await?;

    let response = "Login to Telegram successful!";
    message.respond(response).await.details(response)?;

    Ok(())
}
