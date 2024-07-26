/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client::{OneDriveClient, TelegramMessage};
use crate::error::{Result, ResultExt};

pub async fn show_dir(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    let root_path = onedrive.get_root_path(false).await?;
    let is_temp = onedrive.does_temp_root_path_exist().await;

    let response = if !is_temp {
        format!("Current directory is {}", root_path)
    } else {
        format!("Current directory is {}, and it's temporary.", root_path)
    };
    message.respond(response.as_str()).await.details(response)?;

    Ok(())
}
