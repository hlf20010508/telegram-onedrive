/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;
mod utils;

use grammers_client::types::Message;
use grammers_client::InputMessage;
use std::sync::Arc;
use utils::is_root_path_valid;

use super::utils::cmd_parser;
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::{check_in_group, check_od_login, check_senders};

pub const PATTERN: &str = "/dir";

pub async fn handler(message: Arc<Message>, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_od_login!(message, state);

    let onedrive = &state.onedrive;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /dir
        let root_path = onedrive.get_root_path(false).await?;
        let is_temp = onedrive.does_temp_root_path_exist().await;

        let response = if !is_temp {
            format!("Current directory is {}", root_path)
        } else {
            format!("Current directory is {}, and it's temporary.", root_path)
        };
        message
            .respond(response.as_str())
            .await
            .map_err(|e| Error::respond_error(e, response))?;
    } else if cmd.len() == 2 {
        if cmd[1] == "reset" {
            // /dir reset
            onedrive.reset_root_path().await?;

            let response = format!("Directory reset to default {}", onedrive.default_root_path);
            message
                .respond(response.as_str())
                .await
                .map_err(|e| Error::respond_error(e, response))?;
        } else {
            // dir $root_path
            let root_path = &cmd[1];

            if is_root_path_valid(root_path, message.clone()).await? {
                onedrive.set_root_path(&root_path).await?;

                let response = format!("Directory set to {}", root_path);
                message
                    .respond(response.as_str())
                    .await
                    .map_err(|e| Error::respond_error(e, response))?;
            }
        }
    } else if cmd.len() == 3 {
        if cmd[1] == "temp" {
            if cmd[2] != "cancel" {
                // /dir temp $temp_root_path
                let temp_root_path = &cmd[2];

                if is_root_path_valid(temp_root_path, message.clone()).await? {
                    onedrive.set_temp_root_path(temp_root_path).await?;

                    let response = format!("Temporary directory set to {}", temp_root_path);
                    message
                        .respond(response.as_str())
                        .await
                        .map_err(|e| Error::respond_error(e, response))?;
                }
            } else {
                // /dir temp cancel
                onedrive.clear_temp_root_path().await?;

                let response = format!(
                    "Temporary directory canceled.\nCurrent directory is {}",
                    onedrive.get_root_path(false).await?
                );
                message
                    .respond(response.as_str())
                    .await
                    .map_err(|e| Error::respond_error(e, response))?;
            }
        } else {
            message
                .respond(InputMessage::html(format!(
                    "Unknown sub command for /dir\n{}",
                    docs::USAGE
                )))
                .await
                .map_err(|e| Error::context(e, "failed to respond sub command error for /dir"))?;
        }
    } else {
        message
            .respond(InputMessage::html(format!(
                "Unknown command for /dir\n{}",
                docs::USAGE
            )))
            .await
            .map_err(|e| Error::context(e, "failed to respond command error for /dir"))?;
    }

    Ok(())
}
