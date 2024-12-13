/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{tasks, transfer::multi_parts_uploader_from_url, Progress};
use anyhow::Result;
use std::sync::Arc;

pub async fn handler(task: tasks::Model, progress: Arc<Progress>) -> Result<()> {
    let filename = multi_parts_uploader_from_url(&task, progress.clone()).await?;

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
