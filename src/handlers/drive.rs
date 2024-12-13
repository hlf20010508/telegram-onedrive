/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    docs::{format_help, format_unknown_command_help},
    utils::text::cmd_parser,
};
use crate::{
    auth_server, client::OneDriveClient, handlers::auth::authorize_onedrive,
    message::TelegramMessage, state::AppState,
};
use anyhow::{anyhow, Context, Result};
use grammers_client::InputMessage;
use proc_macros::{check_in_group, check_senders};

pub const PATTERN: &str = "/drive";

#[check_senders]
#[check_in_group]
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
                .context("account index should be integer")?
                - 1;

            set_drive(onedrive, message, index).await?;
        }
    } else if cmd.len() == 3 {
        if cmd[1] == "logout" {
            // /drive logout $index
            let index = cmd[2]
                .parse::<usize>()
                .context("account index should be integer")?
                - 1;

            logout_drive(onedrive, message, index).await?;
        } else {
            return Err(anyhow!("sub command error")).context(format_unknown_command_help(PATTERN));
        }
    } else {
        return Err(anyhow!("command error")).context(format_unknown_command_help(PATTERN));
    }

    Ok(())
}

async fn show_drive(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    let usernames = onedrive.get_usernames().await?;
    if let Some(current_username) = onedrive.get_current_username().await? {
        if !usernames.is_empty() {
            let response = {
                let mut response = format!("Current account is {}", current_username);

                if usernames.len() > 1 {
                    response.insert(0, '\n');
                    for i in (1..=usernames.len()).rev() {
                        response.insert_str(0, &format!("{}. {}\n", i, usernames[i - 1]));
                    }
                }

                response
            };
            message.respond(response.as_str()).await.context(response)?;

            return Ok(());
        }
    }

    let response = "No account found.";
    message.respond(response).await.context(response)?;

    Ok(())
}

async fn add_drive(message: TelegramMessage, state: AppState) -> Result<()> {
    let (_, rx, _server_abort_handle) = auth_server::spawn().await?;
    authorize_onedrive(message, state, true, rx).await?;

    Ok(())
}

async fn logout_current_drive(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    let current_username = onedrive
        .get_current_username()
        .await?
        .ok_or_else(|| anyhow!("no onedrive account is logged in"))?;

    onedrive.logout(None).await?;

    let response = {
        let mut response = format!(
            "OneDrive account {} logged out successfully.",
            current_username
        );

        if let Some(current_username) = onedrive.get_current_username().await? {
            response.push_str(&format!("\n\nCurrent account is {}", current_username));
        }

        response
    };

    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}

async fn set_drive(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    index: usize,
) -> Result<()> {
    let current_username = onedrive
        .get_current_username()
        .await?
        .ok_or_else(|| anyhow!("no onedrive account is logged in"))?;

    let usernames = onedrive.get_usernames().await?;

    let selected_username = usernames
        .get(index)
        .ok_or_else(|| anyhow!("account index out of range"))?;

    onedrive.change_account(selected_username).await?;

    if current_username == *selected_username {
        let response = "Same account, nothing to change.";
        message.respond(response).await.context(response)?;
    } else {
        let response = format!(
            "Changed account from\n{}\nto\n{}",
            current_username, selected_username
        );
        message.respond(response.as_str()).await.context(response)?;
    }

    Ok(())
}

async fn logout_drive(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    index: usize,
) -> Result<()> {
    let usernames = onedrive.get_usernames().await?;

    let selected_username = usernames
        .get(index)
        .ok_or_else(|| anyhow!("account index out of range"))?;

    onedrive.logout(Some(selected_username.clone())).await?;

    let response = {
        let mut response = format!(
            "OneDrive account {} logged out successfully.",
            selected_username
        );

        if let Some(current_username) = onedrive.get_current_username().await? {
            response.push_str(&format!("\n\nCurrent account is {}", current_username));
        }

        response
    };

    message.respond(response.as_str()).await.context(response)?;

    Ok(())
}
