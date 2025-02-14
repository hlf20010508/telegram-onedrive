/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::state::AppState;
use anyhow::{Context, Result};
use grammers_client::types::{
    media::Uploaded,
    photo_sizes::{PhotoSize, VecExt},
};
use std::io::Cursor;

pub async fn upload_thumb(state: AppState, thumbs: Vec<PhotoSize>) -> Result<Option<Uploaded>> {
    let uploaded = match thumbs.largest() {
        Some(thumb) => {
            let mut download = state.telegram_user.iter_download(thumb);

            let mut buffer = Vec::new();
            while let Some(chunk) = download
                .next()
                .await
                .context("failed to download chunk for thumb")?
            {
                buffer.extend(chunk);
            }

            let size = buffer.len();
            let mut stream = Cursor::new(buffer);
            let uploaded = state
                .telegram_bot
                .upload_stream(&mut stream, size, "thumb.jpg".to_string())
                .await
                .context("thumb")?;

            Some(uploaded)
        }
        None => None,
    };

    Ok(uploaded)
}
