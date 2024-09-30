/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    client::TelegramClient,
    error::{Error, Result},
};
use grammers_client::types::{
    media::Uploaded,
    photo_sizes::{PhotoSize, VecExt},
    Downloadable,
};
use proc_macros::{add_context, add_trace};
use std::io::Cursor;

#[add_context]
#[add_trace]
pub async fn upload_thumb(
    client: &TelegramClient,
    thumbs: Vec<PhotoSize>,
) -> Result<Option<Uploaded>> {
    let uploaded = match thumbs.largest() {
        Some(thumb) => {
            let downloadable = Downloadable::PhotoSize(thumb.clone());
            let mut download = client.iter_download(&downloadable);

            let mut buffer = Vec::new();
            while let Some(chunk) = download
                .next()
                .await
                .map_err(|e| Error::new("failed to download chunk for thumb").raw(e))?
            {
                buffer.extend(chunk);
            }

            let size = buffer.len();
            let mut stream = Cursor::new(buffer);
            let uploaded = client
                .upload_stream(&mut stream, size, "thumb.jpg".to_string())
                .await
                .context("thumb")?;

            Some(uploaded)
        }
        None => None,
    };

    Ok(uploaded)
}
