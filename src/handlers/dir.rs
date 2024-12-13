/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    docs::{format_help, format_unknown_command_help},
    utils::{text::cmd_parser, validate_root_path},
};
use crate::{client::OneDriveClient, message::TelegramMessage, state::AppState};
use anyhow::{anyhow, Context, Result};
use grammers_client::InputMessage;
use proc_macros::{check_in_group, check_od_login, check_senders};

pub const PATTERN: &str = "/dir";

#[check_od_login]
#[check_senders]
#[check_in_group]
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
            return Err(anyhow!("sub command error")).context(format_unknown_command_help(PATTERN));
        }
    } else {
        return Err(anyhow!("command error")).context(format_unknown_command_help(PATTERN));
    }

    Ok(())
}

async fn show_dir(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    let root_path = onedrive.get_root_path(false).await?;
    let is_temp = onedrive.does_temp_root_path_exist().await;

    let response = if is_temp {
        format!("Current directory is {}, and it's temporary.", root_path)
    } else {
        format!("Current directory is {}", root_path)
    };
    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}

async fn reset_dir(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    onedrive.reset_root_path().await?;

    let response = format!("Directory reset to default {}", onedrive.default_root_path);
    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}

async fn set_dir(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    root_path: &str,
) -> Result<()> {
    validate_root_path(root_path).await?;

    onedrive.set_root_path(root_path).await?;

    let response = format!("Directory set to {}", root_path);
    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}

async fn cancel_temp_dir(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    onedrive.clear_temp_root_path().await?;

    let response = format!(
        "Temporary directory canceled.\nCurrent directory is {}",
        onedrive.get_root_path(false).await?
    );
    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}

async fn set_temp_dir(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    temp_root_path: &str,
) -> Result<()> {
    validate_root_path(temp_root_path).await?;

    onedrive.set_temp_root_path(temp_root_path).await?;

    let response = format!("Temporary directory set to {}", temp_root_path);
    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}
