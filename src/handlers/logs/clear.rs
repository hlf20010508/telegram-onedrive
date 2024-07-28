/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::add_trace;
use tokio::fs;

use crate::env::LOG_PATH;
use crate::error::{Error, Result};
use crate::message::TelegramMessage;

#[add_trace(context)]
pub async fn clear_logs(message: TelegramMessage) -> Result<()> {
    fs::remove_file(LOG_PATH)
        .await
        .map_err(|e| Error::new_sys_io(e, "failed to remove log file"))?;

    let response = "Logs cleared.";
    message.respond(response).await.details(response)?;

    Ok(())
}
