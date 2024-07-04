/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::sync::Arc;

use crate::error::{Error, Result};

pub async fn is_root_path_valid(root_path: &str, message: Arc<Message>) -> Result<bool> {
    if !root_path.starts_with('/') {
        let response = "directory path should start with /";
        message
            .reply(response)
            .await
            .map_err(|e| Error::respond_error(e, response))?;

        Ok(false)
    } else {
        Ok(true)
    }
}
