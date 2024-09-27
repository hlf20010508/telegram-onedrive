/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace};
use tokio::fs;

use crate::env::LOGS_PATH;
use crate::error::{Error, Result};
use crate::message::TelegramMessage;

#[add_context]
#[add_trace]
pub async fn clear_logs(message: TelegramMessage) -> Result<()> {
    while let Some(entry) = fs::read_dir(LOGS_PATH)
        .await
        .map_err(|e| Error::new_sys_io(e, "failed to read logs dir"))?
        .next_entry()
        .await
        .map_err(|e| Error::new_sys_io(e, "failed to read next entry in logs dir"))?
    {
        fs::remove_file(entry.path()).await.map_err(|e| {
            Error::new_sys_io(e, "failed to remove log file")
                .details(entry.path().to_string_lossy())
        })?;
    }

    let response = "Logs cleared.";
    message.respond(response).await.details(response)?;

    Ok(())
}
