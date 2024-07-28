/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod onedrive;
mod telegram;

pub use onedrive::authorize_onedrive;
use proc_macros::add_trace;
pub use telegram::login_to_telegram;

use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::{auth_server, check_in_group, check_senders};

pub const PATTERN: &str = "/auth";

#[add_trace(context)]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);

    let env = &state.env;

    let _server_abort_handle = auth_server::spawn(env).await?;

    login_to_telegram(message.clone(), state.clone()).await?;

    authorize_onedrive(message, state.clone(), false).await?;

    let onedrive = &state.onedrive;
    onedrive.set_current_user().await?;

    Ok(())
}
