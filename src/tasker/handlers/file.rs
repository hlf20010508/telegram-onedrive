/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{tasks, transfer::multi_parts_uploader_from_tg_file, Progress};
use crate::{error::Result, state::AppState};
use proc_macros::{add_context, add_trace};
use std::sync::Arc;

#[add_context]
#[add_trace]
pub async fn handler(task: tasks::Model, progress: Arc<Progress>, state: AppState) -> Result<()> {
    let filename = multi_parts_uploader_from_tg_file(&task, progress.clone(), state).await?;

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
