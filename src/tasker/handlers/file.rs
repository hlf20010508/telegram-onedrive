/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{tasks, transfer::multi_parts_uploader_from_tg_file};
use crate::{error::TaskAbortError, state::AppState};
use anyhow::Result;
use tokio_util::sync::CancellationToken;

pub async fn handler(
    task: tasks::Model,
    cancellation_token: CancellationToken,
    state: AppState,
) -> Result<()> {
    let filename =
        match multi_parts_uploader_from_tg_file(&task, cancellation_token, state.clone()).await {
            Ok(filename) => filename,
            Err(e) => {
                if e.downcast_ref::<TaskAbortError>().is_some() {
                    return Ok(());
                }
                return Err(e);
            }
        };

    state
        .task_session
        .update_filename(task.id, &filename)
        .await?;

    Ok(())
}
