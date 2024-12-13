/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{auth_server, message::TelegramMessage, state::AppState};
use anyhow::{Context, Result};
use proc_macros::{check_in_group, check_senders};
use tokio::sync::mpsc::Receiver;

pub const PATTERN: &str = "/auth";

#[check_senders]
#[check_in_group]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let (rx_tg, rx_od, _server_abort_handle) = auth_server::spawn().await?;

    login_to_telegram(message.clone(), state.clone(), rx_tg).await?;

    authorize_onedrive(message, state.clone(), false, rx_od).await?;

    let onedrive = &state.onedrive;
    onedrive.set_current_user().await?;

    Ok(())
}

pub async fn login_to_telegram(
    message: TelegramMessage,
    state: AppState,
    rx: Receiver<String>,
) -> Result<()> {
    let telegram_user = &state.telegram_user;

    telegram_user.login(message.clone(), rx).await?;

    let response = "Login to Telegram successful!";
    message.respond(response).await.context(response)?;

    Ok(())
}

pub async fn authorize_onedrive(
    message: TelegramMessage,
    state: AppState,
    should_add: bool,
    rx: Receiver<String>,
) -> Result<()> {
    let onedrive = &state.onedrive;

    onedrive.login(message.clone(), should_add, rx).await?;

    let response = "OneDrive authorization successful!";
    message.respond(response).await.context(response)?;

    Ok(())
}
