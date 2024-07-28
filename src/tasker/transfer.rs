/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Downloadable;
use onedrive_api::resource::DriveItem;
use onedrive_api::UploadSession;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use super::{tasks, Progress};
use crate::client::ext::chat_from_hex;
use crate::error::{Error, Result, ResultExt};
use crate::state::AppState;
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

    let url = url.clone().ok_or_else(|| Error::new("url is none"))?;

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
        .map_err(|e| Error::new_http_request(e, "failed to send head request for /url"))?;

    let max_retries = 5;

    const PART_SIZE: usize = 3276800;

    loop {
        let mut buffer = Vec::with_capacity(PART_SIZE);

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| Error::new_http_request(e, "failed to get chunk"))?
        {
            buffer.extend_from_slice(&chunk);

            if buffer.len() >= PART_SIZE {
                break;
            }
        }

        _upload_response = upload_file(
            &upload_session,
            &buffer,
            current_length,
            total_length,
            &http_client,
            max_retries,
        )
        .await?;

        current_length += buffer.len() as u64;
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

pub async fn multi_parts_uploader_from_tg_file(
    tasks::Model {
        id,
        upload_url,
        current_length,
        total_length,
        chat_user_hex,
        message_id,
        message_id_forward,
        ..
    }: &tasks::Model,
    progress: Arc<Progress>,
    state: AppState,
) -> Result<String> {
    let http_client = get_http_client().await?;

    let upload_session = UploadSession::from_upload_url(upload_url);

    let mut current_length = current_length.to_owned() as u64;
    let total_length = total_length.to_owned() as u64;

    progress
        .set_current_length(id.to_owned(), current_length)
        .await?;

    let mut upload_response = None;

    let telegram_user = &state.telegram_user;

    let chat = chat_from_hex(chat_user_hex)?;

    let message_id = match message_id_forward {
        Some(message_id) => message_id,
        None => message_id,
    };

    let message = telegram_user
        .get_message(chat, *message_id)
        .await
        .context("multi_parts_uploader_from_tg_file")?;
    let media = message
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;
    let mut download = telegram_user.iter_download(&Downloadable::Media(media));

    let max_retries = 5;

    while let Some(chunk) = download.next().await.map_err(|e| {
        Error::new_telegram_invocation(e, "failed to get next chunk from tg file downloader")
    })? {
        upload_response = upload_file(
            &upload_session,
            &chunk,
            current_length,
            total_length,
            &http_client,
            max_retries,
        )
        .await?;

        current_length += chunk.len() as u64;
        progress
            .set_current_length(id.to_owned(), current_length)
            .await?;

        if current_length >= total_length {
            break;
        }
    }

    let filename = upload_response
        .ok_or_else(|| Error::new("failed to get drive item after upload"))?
        .name
        .ok_or_else(|| Error::new("drive item name not found"))?;

    if message_id_forward.is_some() {
        message.delete().await.context("forwarded message")?;
    }

    Ok(filename)
}

async fn upload_file(
    upload_session: &UploadSession,
    buffer: &[u8],
    current_length: u64,
    total_length: u64,
    http_client: &reqwest::Client,
    max_retries: i32,
) -> Result<Option<DriveItem>> {
    let mut upload_response = None;

    let mut tries = 0;

    loop {
        tries += 1;

        let result = upload_session
            .upload_part(
                buffer.to_owned(),
                Range {
                    start: current_length,
                    end: current_length + buffer.len() as u64,
                },
                total_length,
                http_client,
            )
            .await;

        match result {
            Ok(response) => {
                upload_response = response;

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
                            if upload_response.is_some() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if tries < max_retries {
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    continue;
                } else {
                    return Err(Error::new_onedrive(e, "failed to upload part"));
                }
            }
        }
    }

    Ok(upload_response)
}
