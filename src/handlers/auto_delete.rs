/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{message::TelegramMessage, state::AppState};
use anyhow::{Context, Result};
use proc_macros::{check_in_group, check_senders};
use std::sync::atomic::Ordering;

pub const PATTERN: &str = "/autoDelete";

#[check_senders]
#[check_in_group]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let should_auto_delete = state.should_auto_delete.load(Ordering::Acquire);

    state
        .should_auto_delete
        .store(!should_auto_delete, Ordering::Release);

    if should_auto_delete {
        let response = "Bot won't auto delete message.";
        message.respond(response).await.context(response)?;
    } else {
        let response = "Bot will auto delete message.";
        message.respond(response).await.context(response)?;
    }

    Ok(())
}
