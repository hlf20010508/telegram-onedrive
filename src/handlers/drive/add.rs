/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace};

use crate::auth_server;
use crate::error::Result;
use crate::handlers::auth::authorize_onedrive;
use crate::message::TelegramMessage;
use crate::state::AppState;

#[add_context]
#[add_trace]
pub async fn add_drive(message: TelegramMessage, state: AppState) -> Result<()> {
    let env = &state.env;

    let _server_abort_handle = auth_server::spawn(env).await?;
    authorize_onedrive(message, state, true).await?;

    Ok(())
}
