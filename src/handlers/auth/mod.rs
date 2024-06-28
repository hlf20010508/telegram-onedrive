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
use crate::{auth_server, check_in_group, check_senders};

pub const PATTERN: &str = "/auth";

pub async fn handler(message: Arc<Message>, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);

    let _server_abort_handle = auth_server::spawn().await?;

    let env = &state.env;
    let telegram_user = &state.telegram_user;

    message
        .respond("Logining into Telegram...")
        .await
        .map_err(|e| Error::context(e, "failed to respond message in auth"))?;

    if let Err(e) = telegram_user.login(message.clone(), env).await {
        return Err(e);
    }

    message
        .respond("Login to Telegram successful!")
        .await
        .map_err(|e| Error::context(e, "failed to respond telegram login suscessful"))?;

    Ok(())
}
