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
        current_length,
        total_length,
        ..
    }: &tasks::Model,
    progress: Arc<Progress>,
) -> Result<String> {
    let http_client = get_http_client().await?;

    let upload_session = UploadSession::from_upload_url(upload_url);

    let mut current_length = current_length.to_owned() as u64;
    let total_length = total_length.to_owned() as u64;

    progress
        .set_current_length(id.to_owned(), current_length)
        .await?;

    let mut _upload_response = None;

    let mut response = http_client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::context(e, "failed to send head request for /url"))?;

    let max_retries = 5;

    loop {
        let mut buffer = Vec::with_capacity(PART_SIZE);

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

            let result = upload_session
                .upload_part(
                    buffer.clone(),
                    Range {
                        start: current_length,
                        end: current_length + buffer_length,
                    },
                    total_length,
                    &http_client,
                )
                .await;

            match result {
                Ok(response) => {
                    _upload_response = response;

                    break;
                }
                Err(e) => {
                    if let Some(status_code) = e.status_code() {
                        match status_code.as_u16() {
                            // 408: Request Timeout
                            // 500: Internal Server Error
                            // 502: Bad Gateway
                            // 503: Service Unavailable
                            // 504: Gateway Timeout
                            408 | 500 | 502 | 503 | 504 if tries < max_retries => {}
                            // 409: Conflict, probably caused by rename, too many files with the same name uploaded at once
                            // 404: Not Found, probably because the item has already been uploaded
                            // 416: Requested Range Not Satisfiable, probably because the fragment has already been received
                            409 | 404 | 416 => {
                                break;
                            }
                            _ => {}
                        }
                    }

                    if tries < max_retries {
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        continue;
                    } else {
                        return Err(Error::context(e, "failed to upload part"));
                    }
                }
            }
        }

        current_length += buffer_length;
        progress
            .set_current_length(id.to_owned(), current_length)
            .await?;

        if current_length >= total_length {
            break;
        }
    }

    let filename = _upload_response
        .ok_or_else(|| Error::new("failed to get drive item after upload"))?
        .name
        .ok_or_else(|| Error::new("drive item name not found"))?;

    Ok(filename)
}
