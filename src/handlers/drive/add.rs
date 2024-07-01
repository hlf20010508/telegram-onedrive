/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::sync::Arc;

use crate::auth_server;
use crate::error::Result;
use crate::handlers::auth::authorize_onedrive;
use crate::state::AppState;

pub async fn add_drive(message: Arc<Message>, state: AppState) -> Result<()> {
    let env = &state.env;

    let _server_abort_handle = auth_server::spawn(env).await?;
    authorize_onedrive(message, state, true).await?;

    Ok(())
}
