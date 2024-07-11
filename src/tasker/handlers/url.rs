/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use path_slash::PathBufExt;
use std::path::Path;
use std::sync::Arc;

use super::transfer::multi_parts_uploader_from_url;
use super::{tasks, Progress};
use crate::error::{Error, Result};

pub async fn handler(task: tasks::Model, message: Message, progress: Arc<Progress>) -> Result<()> {
    let filename = multi_parts_uploader_from_url(&task, progress.clone()).await?;

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
