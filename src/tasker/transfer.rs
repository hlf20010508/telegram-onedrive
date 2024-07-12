/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use onedrive_api::resource::DriveItem;
use onedrive_api::UploadSession;

use super::{tasks, Progress};
use crate::error::{Error, Result};
use crate::tasker::var::PART_SIZE;
use crate::utils::get_http_client;

pub async fn multi_parts_uploader_from_url(
    tasks::Model {
        id,
        url,
        upload_url,
        total_length,
        ..
    }: &tasks::Model,
    progress: Arc<Progress>,
) -> Result<String> {
    let http_client = get_http_client().await?;

    let upload_session = UploadSession::from_upload_url(upload_url);
    let upload_session_meta = upload_session
        .get_meta(&http_client)
        .await
        .map_err(|e| Error::context(e, "failed to get upload session meta"))?;

    let mut current_length = {
        match upload_session_meta.next_expected_ranges.get(0) {
            Some(range) => range.start,
            None => 0,
        }
    };

    progress
        .set_current_length(id.to_owned(), current_length)
        .await?;

    let mut buffer = Vec::with_capacity(PART_SIZE);

    let mut _upload_response = None;

    let mut response = http_client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::context(e, "failed to send head request for /url"))?;

    let max_retries = 5;

    loop {
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| Error::context(e, "failed to get chunk"))?
        {
            buffer.extend_from_slice(&chunk);

            if buffer.len() >= PART_SIZE {
                break;
            }
        }

        let buffer_length = buffer.len() as u64;

        let mut tries = 0;

        loop {
            tries += 1;

            let result = upload_buffer(
                &upload_session,
                &mut buffer,
                current_length,
                total_length.to_owned() as u64,
                &http_client,
            )
            .await;

            match result {
                Ok(response) => {
                    _upload_response = response;

                    break;
                }
                Err(e) => {
                    if tries >= max_retries {
                        return Err(Error::context(e, "failed to upload part"));
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        current_length += buffer_length;
        progress
            .set_current_length(id.to_owned(), current_length)
            .await?;
        buffer.clear();

        if current_length >= total_length.to_owned() as u64 {
            break;
        }
    }

    let filename = _upload_response
        .ok_or_else(|| Error::new("failed to get drive item after upload"))?
        .name
        .ok_or_else(|| Error::new("drive item name not found"))?;

    Ok(filename)
}

async fn upload_buffer(
    upload_session: &UploadSession,
    buffer: &Vec<u8>,
    start_offset: u64,
    total_length: u64,
    http_client: &reqwest::Client,
) -> Result<Option<DriveItem>> {
    let upload_response = upload_session
        .upload_part(
            buffer.clone(),
            Range {
                start: start_offset,
                end: start_offset + buffer.len() as u64,
            },
            total_length,
            http_client,
        )
        .await
        .map_err(|e| Error::context(e, "failed to upload part"))?;

    Ok(upload_response)
}
