/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod onedrive;
mod telegram;

pub use onedrive::authorize_onedrive;
use proc_macros::{add_context, add_trace, check_in_group, check_senders};
pub use telegram::login_to_telegram;

use crate::auth_server;
use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;

pub const PATTERN: &str = "/auth";

#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let env = &state.env;

    let _server_abort_handle = auth_server::spawn(env).await?;

    login_to_telegram(message.clone(), state.clone()).await?;

    authorize_onedrive(message, state.clone(), false).await?;

    let onedrive = &state.onedrive;
    onedrive.set_current_user().await?;

    Ok(())
}
