/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::TelegramClient;
use anyhow::{Context, Result};
use grammers_client::{
    client::files::DownloadIter,
    types::{media::Uploaded, Downloadable},
};
use std::path::Path;
use tokio::io::AsyncRead;

impl TelegramClient {
    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<Uploaded> {
        tracing::info!("uploading file: {}", path.as_ref().to_string_lossy());

        self.raw()
            .upload_file(path)
            .await
            .context("failed to upload file")
    }

    pub async fn upload_stream<S: AsyncRead + Unpin>(
        &self,
        stream: &mut S,
        size: usize,
        name: String,
    ) -> Result<Uploaded> {
        tracing::info!("uploading stream: {} size: {}", name, size);

        self.raw()
            .upload_stream(stream, size, name)
            .await
            .context("failed to upload stream")
    }

    pub fn iter_download<D: Downloadable>(&self, downloadable: &D) -> DownloadIter {
        self.raw().iter_download(downloadable)
    }
}
