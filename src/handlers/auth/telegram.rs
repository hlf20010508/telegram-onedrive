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

pub async fn login_to_telegram(message: Arc<Message>, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let env = &state.env;

    telegram_user.login(message.clone(), env).await?;

    let response = "Login to Telegram successful!";
    message
        .respond(response)
        .await
        .map_err(|e| Error::respond_error(e, response))?;

    Ok(())
}
