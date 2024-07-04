/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod add;
mod docs;
mod logout;
mod set;
mod show;

use grammers_client::types::Message;
use grammers_client::InputMessage;
use std::sync::Arc;

use add::add_drive;
use logout::{logout_current_drive, logout_drive};
use set::set_drive;
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
        } else if cmd[1] == "logout" {
            // /drive logout
            logout_current_drive(onedrive, message).await?;
        } else {
            // /drive $index
            let index = cmd[1]
                .parse::<usize>()
                .map_err(|e| Error::context(e, "account index should be integer"))?
                - 1;

            set_drive(onedrive, message, index).await?;
        }
    } else if cmd.len() == 3 {
        if cmd[1] == "logout" {
            // /drive logout $index
            let index = cmd[2]
                .parse::<usize>()
                .map_err(|e| Error::context(e, "account index should be integer"))?
                - 1;

            logout_drive(onedrive, message, index).await?;
        } else {
            message
                .respond(InputMessage::html(format!(
                    "Unknown sub command for /drive\n{}",
                    docs::USAGE
                )))
                .await
                .map_err(|e| Error::context(e, "failed to respond sub command error for /drive"))?;
        }
    } else {
        message
            .respond(InputMessage::html(format!(
                "Unknown command for /drive\n{}",
                docs::USAGE
            )))
            .await
            .map_err(|e| Error::context(e, "failed to respond command error for /drive"))?;
    }

    Ok(())
}
