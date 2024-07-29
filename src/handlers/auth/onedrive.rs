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
pub async fn authorize_onedrive(
    message: TelegramMessage,
    state: AppState,
    should_add: bool,
) -> Result<()> {
    let onedrive = &state.onedrive;
    let env = &state.env;

    onedrive.login(message.clone(), env, should_add).await?;

    let response = "OneDrive authorization successful!";
    message.respond(response).await.details(response)?;

    Ok(())
}
