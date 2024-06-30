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

    let env = &state.env;

    let _server_abort_handle = auth_server::spawn(&env).await?;

    {
        let telegram_user = &state.telegram_user;

        telegram_user.login(message.clone(), env).await?;

        let response = "Login to Telegram successful!";
        message
            .respond(response)
            .await
            .map_err(|e| Error::respond_error(e, response))?;
    }

    {
        let onedrive = &state.onedrive;

        onedrive.login(message.clone(), env).await?;

        let response = "OneDrive authorization successful!";
        message
            .respond(response)
            .await
            .map_err(|e| Error::respond_error(e, response))?;
    }

    Ok(())
}
