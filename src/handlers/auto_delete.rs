/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::add_trace;
use std::sync::atomic::Ordering;

use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::{check_in_group, check_senders};

pub const PATTERN: &str = "/autoDelete";

#[add_trace(context)]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);

    let should_auto_delete = state.should_auto_delete.load(Ordering::Acquire);

    state
        .should_auto_delete
        .store(!should_auto_delete, Ordering::Release);

    if !should_auto_delete {
        let response = "Bot will auto delete message.";
        message.respond(response).await.details(response)?;
    } else {
        let response = "Bot won't auto delete message.";
        message.respond(response).await.details(response)?;
    }

    Ok(())
}
