/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace};
use std::sync::Arc;

use super::transfer::multi_parts_uploader_from_url;
use super::{tasks, Progress};
use crate::error::Result;
use crate::state::AppState;

#[add_context]
#[add_trace]
pub async fn handler(task: tasks::Model, progress: Arc<Progress>, _: AppState) -> Result<()> {
    let filename = multi_parts_uploader_from_url(&task, progress.clone()).await?;

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
