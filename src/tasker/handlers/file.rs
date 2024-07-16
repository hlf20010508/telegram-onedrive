/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::Arc;

use super::transfer::multi_parts_uploader_from_tg_file;
use super::{tasks, Progress};
use crate::error::Result;
use crate::state::AppState;

pub async fn handler(task: tasks::Model, progress: Arc<Progress>, state: AppState) -> Result<()> {
    let filename = multi_parts_uploader_from_tg_file(&task, progress.clone(), state).await?;

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
