/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{tasks, transfer::multi_parts_uploader_from_tg_file};
use crate::state::AppState;
use anyhow::Result;

pub async fn handler(task: tasks::Model, state: AppState) -> Result<()> {
    let filename = multi_parts_uploader_from_tg_file(&task, state.clone()).await?;

    state
        .task_session
        .update_filename(task.id, &filename)
        .await?;

    Ok(())
}
