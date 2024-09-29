/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    auth_server, error::Result, handlers::auth::authorize_onedrive, message::TelegramMessage,
    state::AppState,
};
use proc_macros::{add_context, add_trace};

#[add_context]
#[add_trace]
pub async fn add_drive(message: TelegramMessage, state: AppState) -> Result<()> {
    let _server_abort_handle = auth_server::spawn().await?;
    authorize_onedrive(message, state, true).await?;

    Ok(())
}
