/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod add;
mod logout;
mod set;
mod show;

use super::{
    docs::{format_help, format_unknown_command_help},
    utils::cmd_parser,
};
use crate::{
    error::{Error, Result},
    message::TelegramMessage,
    state::AppState,
};
use add::add_drive;
use grammers_client::InputMessage;
use logout::{logout_current_drive, logout_drive};
use proc_macros::{add_context, add_trace, check_in_group, check_senders};
use set::set_drive;
use show::show_drive;

pub const PATTERN: &str = "/drive";

#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
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
        } else if cmd[1] == "help" {
            // /drive help
            message
                .respond(InputMessage::html(format_help(PATTERN)))
                .await
                .context("help")?;
        } else {
            // /drive $index
            let index = cmd[1]
                .parse::<usize>()
                .map_err(|e| Error::new("account index should be integer").raw(e))?
                - 1;

            set_drive(onedrive, message, index).await?;
        }
    } else if cmd.len() == 3 {
        if cmd[1] == "logout" {
            // /drive logout $index
            let index = cmd[2]
                .parse::<usize>()
                .map_err(|e| Error::new("account index should be integer").raw(e))?
                - 1;

            logout_drive(onedrive, message, index).await?;
        } else {
            message
                .reply(InputMessage::html(format_unknown_command_help(PATTERN)))
                .await
                .context("sub command error")?;
        }
    } else {
        message
            .reply(InputMessage::html(format_unknown_command_help(PATTERN)))
            .await
            .context("command error")?;
    }

    Ok(())
}
