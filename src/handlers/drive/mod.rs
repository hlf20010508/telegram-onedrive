/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod add;
mod show;

use grammers_client::types::Message;
use std::sync::Arc;

use add::add_drive;
use show::show_drive;

use super::utils::cmd_parser;
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::{check_in_group, check_od_login, check_senders};

pub const PATTERN: &str = "/drive";

pub async fn handler(message: Arc<Message>, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_od_login!(message, state);

    let onedrive = &state.onedrive;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /drive
        show_drive(onedrive, message).await?;
    } else if cmd.len() == 2 {
        if cmd[1] == "add" {
            // /drive add
            add_drive(message, state.clone()).await?;
        }
    }

    Ok(())
}
