/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    env::LOGS_PATH,
    error::{Error, Result},
    message::TelegramMessage,
};
use proc_macros::{add_context, add_trace};
use tokio::fs;

#[add_context]
#[add_trace]
pub async fn clear_logs(message: TelegramMessage) -> Result<()> {
    while let Some(entry) = fs::read_dir(LOGS_PATH)
        .await
        .map_err(|e| Error::new("failed to read logs dir").raw(e))?
        .next_entry()
        .await
        .map_err(|e| Error::new("failed to read next entry in logs dir").raw(e))?
    {
        fs::remove_file(entry.path()).await.map_err(|e| {
            Error::new("failed to remove log file")
                .raw(e)
                .details(entry.path().to_string_lossy())
        })?;
    }

    let response = "Logs cleared.";
    message.respond(response).await.details(response)?;

    Ok(())
}
