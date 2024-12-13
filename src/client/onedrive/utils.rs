/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::invalid_name::INVALID_FOLDER_DIR;
use anyhow::{anyhow, Result};

pub fn validate_root_path(path: &str) -> Result<()> {
    if path == INVALID_FOLDER_DIR || path.starts_with(&format!("{}/", INVALID_FOLDER_DIR)) {
        return Err(anyhow!(format!(
            "root folder should not be {} according restrictions",
            INVALID_FOLDER_DIR
        )));
    }

    Ok(())
}
