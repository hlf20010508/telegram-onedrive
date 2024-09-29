/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::invalid_name::INVALID_FOLDER_DIR;
use crate::error::{Error, Result};
use proc_macros::{add_context, add_trace};

#[add_context]
#[add_trace]
pub fn validate_root_path(path: &str) -> Result<()> {
    if path == INVALID_FOLDER_DIR || path.starts_with(&format!("{}/", INVALID_FOLDER_DIR)) {
        return Err(Error::new(format!(
            "root folder should not be {} according restrictions",
            INVALID_FOLDER_DIR
        )));
    }

    Ok(())
}
