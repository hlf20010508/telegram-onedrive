/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{tasks, transfer::multi_parts_uploader_from_url, Progress};
use crate::{
    client::utils::chat_from_hex,
    error::{Error, Result},
    state::AppState,
};
use proc_macros::{add_context, add_trace};
use std::sync::Arc;

#[add_context]
#[add_trace]
pub async fn handler(task: tasks::Model, progress: Arc<Progress>, state: AppState) -> Result<()> {
    let filename = multi_parts_uploader_from_url(&task, progress.clone()).await?;

    let chat = chat_from_hex(&task.chat_user_hex)?;
    state
        .task_session
        .aborters
        .write()
        .await
        .remove(&(chat.id, task.message_id))
        .ok_or_else(|| Error::new("task aborter not found"))?;

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
