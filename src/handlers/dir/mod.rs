/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod reset;
mod set;
mod show;
mod utils;

use super::{
    docs::{format_help, format_unknown_command_help},
    utils::cmd_parser,
};
use crate::{error::Result, message::TelegramMessage, state::AppState};
use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace, check_in_group, check_od_login, check_senders};
use reset::{cancel_temp_dir, reset_dir};
use set::{set_dir, set_temp_dir};
use show::show_dir;

pub const PATTERN: &str = "/dir";

#[check_od_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let onedrive = &state.onedrive;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /dir
        show_dir(onedrive, message).await?;
    } else if cmd.len() == 2 {
        if cmd[1] == "reset" {
            // /dir reset
            reset_dir(onedrive, message).await?;
        } else if cmd[1] == "help" {
            // /dir help
            message
                .respond(InputMessage::html(format_help(PATTERN)))
                .await
                .context("help")?;
        } else {
            // dir $root_path
            let root_path = &cmd[1];
            set_dir(onedrive, message, root_path).await?;
        }
    } else if cmd.len() == 3 {
        if cmd[1] == "temp" {
            if cmd[2] == "cancel" {
                // /dir temp cancel
                cancel_temp_dir(onedrive, message).await?;
            } else {
                // /dir temp $path
                let temp_root_path = &cmd[2];
                set_temp_dir(onedrive, message, temp_root_path).await?;
            }
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
