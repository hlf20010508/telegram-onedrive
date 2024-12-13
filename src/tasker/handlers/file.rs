/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{tasks, transfer::multi_parts_uploader_from_tg_file, Progress};
use crate::{error::TaskAbortError, state::AppState};
use anyhow::Result;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn handler(
    task: tasks::Model,
    progress: Arc<Progress>,
    cancellation_token: CancellationToken,
    state: AppState,
) -> Result<()> {
    let filename =
        match multi_parts_uploader_from_tg_file(&task, progress.clone(), cancellation_token, state)
            .await
        {
            Ok(filename) => filename,
            Err(e) => {
                if e.downcast_ref::<TaskAbortError>().is_some() {
                    return Ok(());
                }
                return Err(e);
            }
        };

    progress.update_filename(task.id, &filename).await?;

    Ok(())
}
