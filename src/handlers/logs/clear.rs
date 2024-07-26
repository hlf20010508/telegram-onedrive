/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::sync::Arc;
use tokio::fs;

use crate::env::LOG_PATH;
use crate::error::{Error, Result};

pub async fn clear_logs(message: Arc<Message>) -> Result<()> {
    fs::remove_file(LOG_PATH)
        .await
        .map_err(|e| Error::new_sys_io(e, "failed to remove log file"))?;

    let response = "Logs cleared.";
    message
        .respond(response)
        .await
        .map_err(|e| Error::respond_error(e, response))?;

    Ok(())
}
