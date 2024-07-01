/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::state::AppState;

pub async fn authorize_onedrive(
    message: Arc<Message>,
    state: AppState,
    should_add: bool,
) -> Result<()> {
    let onedrive = &state.onedrive;
    let env = &state.env;

    onedrive.login(message.clone(), env, should_add).await?;

    let response = "OneDrive authorization successful!";
    message
        .respond(response)
        .await
        .map_err(|e| Error::respond_error(e, response))?;

    Ok(())
}
