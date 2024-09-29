/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::utils::is_root_path_valid;
use crate::{client::OneDriveClient, error::Result, message::TelegramMessage};
use proc_macros::{add_context, add_trace};

#[add_context]
#[add_trace]
pub async fn set_dir(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    root_path: &str,
) -> Result<()> {
    if is_root_path_valid(root_path, message.clone()).await? {
        onedrive.set_root_path(root_path).await?;

        let response = format!("Directory set to {}", root_path);
        message.respond(response.as_str()).await.details(response)?;
    }

    Ok(())
}

#[add_context]
#[add_trace]
pub async fn set_temp_dir(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    temp_root_path: &str,
) -> Result<()> {
    if is_root_path_valid(temp_root_path, message.clone()).await? {
        onedrive.set_temp_root_path(temp_root_path).await?;

        let response = format!("Temporary directory set to {}", temp_root_path);
        message.respond(response.as_str()).await.details(response)?;
    }

    Ok(())
}
