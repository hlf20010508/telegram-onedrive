/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client::TelegramMessage;
use crate::error::{Result, ResultExt};
use crate::state::AppState;

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
