/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::client::files::DownloadIter;
use grammers_client::types::media::Uploaded;
use grammers_client::types::Downloadable;
use proc_macros::{add_context, add_trace};
use std::path::Path;
use tokio::io::AsyncRead;

use super::TelegramClient;

use crate::error::{Error, Result};

impl TelegramClient {
    #[add_context]
    #[add_trace]
    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<Uploaded> {
        self.raw()
            .upload_file(path)
            .await
            .map_err(|e| Error::new_sys_io(e, "failed to upload file"))
    }

    #[add_context]
    #[add_trace]
    pub async fn upload_stream<S: AsyncRead + Unpin>(
        &self,
        stream: &mut S,
        size: usize,
        name: String,
    ) -> Result<Uploaded> {
        self.raw()
            .upload_stream(stream, size, name)
            .await
            .map_err(|e| Error::new_sys_io(e, "failed to upload stream"))
    }

    pub fn iter_download(&self, downloadable: &Downloadable) -> DownloadIter {
        self.raw().iter_download(downloadable)
    }
}
