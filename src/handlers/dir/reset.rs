/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client::{OneDriveClient, TelegramMessage};
use crate::error::{Result, ResultExt};

pub async fn reset_dir(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    onedrive.reset_root_path().await?;

    let response = format!("Directory reset to default {}", onedrive.default_root_path);
    message.respond(response.as_str()).await.details(response)?;

    Ok(())
}

pub async fn cancel_temp_dir(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    onedrive.clear_temp_root_path().await?;

    let response = format!(
        "Temporary directory canceled.\nCurrent directory is {}",
        onedrive.get_root_path(false).await?
    );
    message.respond(response.as_str()).await.details(response)?;

    Ok(())
}
