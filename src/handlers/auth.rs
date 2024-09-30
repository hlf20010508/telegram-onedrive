/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{auth_server, error::Result, message::TelegramMessage, state::AppState};
use proc_macros::{add_context, add_trace, check_in_group, check_senders};

pub const PATTERN: &str = "/auth";

#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let _server_abort_handle = auth_server::spawn().await?;

    login_to_telegram(message.clone(), state.clone()).await?;

    authorize_onedrive(message, state.clone(), false).await?;

    let onedrive = &state.onedrive;
    onedrive.set_current_user().await?;

    Ok(())
}

#[add_context]
#[add_trace]
pub async fn authorize_onedrive(
    message: TelegramMessage,
    state: AppState,
    should_add: bool,
) -> Result<()> {
    let onedrive = &state.onedrive;

    onedrive.login(message.clone(), should_add).await?;

    let response = "OneDrive authorization successful!";
    message.respond(response).await.details(response)?;

    Ok(())
}

#[add_context]
#[add_trace]
pub async fn login_to_telegram(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;

    telegram_user.login(message.clone()).await?;

    let response = "Login to Telegram successful!";
    message.respond(response).await.details(response)?;

    Ok(())
}
