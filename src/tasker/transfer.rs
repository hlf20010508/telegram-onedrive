/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::client::files::MAX_CHUNK_SIZE;
use grammers_client::types::Downloadable;
use onedrive_api::resource::DriveItem;
use onedrive_api::UploadSession;
use proc_macros::{add_context, add_trace};
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use super::{tasks, Progress};
use crate::client::ext::chat_from_hex;
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::utils::get_http_client;

const MAX_RETRIES: i32 = 5;

#[add_context]
#[add_trace]
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
    const PART_SIZE: usize = 3276800;

    let http_client = get_http_client()?;

    let url = url.clone().ok_or_else(|| Error::new("url is none"))?;

    let upload_session = UploadSession::from_upload_url(upload_url);

    let mut current_length = current_length.to_owned() as u64;
    let total_length = total_length.to_owned() as u64;

    progress
        .set_current_length(id.to_owned(), current_length)
        .await?;

    let mut response = http_client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::new_http_request(e, "failed to send head request for /url"))?;

    let upload_response = loop {
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

        let upload_response = upload_file(
            &upload_session,
            &buffer,
            current_length,
            total_length,
            &http_client,
        )
        .await?;

        current_length += buffer.len() as u64;
        progress
            .set_current_length(id.to_owned(), current_length)
            .await?;

        if current_length >= total_length {
            break upload_response;
        }
    };

    let filename = upload_response
        .ok_or_else(|| Error::new("failed to get drive item after upload"))?
        .name
        .ok_or_else(|| Error::new("drive item name not found"))?;

    Ok(filename)
}

#[add_context]
#[add_trace]
pub async fn multi_parts_uploader_from_tg_file(
    tasks::Model {
        id,
        cmd_type,
        upload_url,
        current_length,
        total_length,
        chat_user_hex,
        chat_origin_hex,
        message_id,
        message_id_forward,
        message_id_origin,
        ..
    }: &tasks::Model,
    progress: Arc<Progress>,
    state: AppState,
) -> Result<String> {
    const WORKER_COUNT: i32 = 10;

    let http_client = get_http_client()?;

    let upload_session = UploadSession::from_upload_url(upload_url);

    let mut current_length = current_length.to_owned() as u64;
    let total_length = total_length.to_owned() as u64;

    progress
        .set_current_length(id.to_owned(), current_length)
        .await?;

    let mut upload_response = None;

    let telegram_user = &state.telegram_user;

    let message = match cmd_type {
        tasks::CmdType::File => {
            let chat = chat_from_hex(chat_user_hex)?;

            let message_id = message_id_forward
                .as_ref()
                .map_or(message_id, |message_id| message_id);

            telegram_user.get_message(chat, *message_id).await?
        }
        tasks::CmdType::Link => {
            let chat = chat_from_hex(
                chat_origin_hex
                    .as_ref()
                    .ok_or_else(|| Error::new("chat_origin_hex is None"))?,
            )?;

            let message_id = message_id_origin
                .as_ref()
                .ok_or_else(|| Error::new("message_id_origin is None"))?;

            telegram_user.get_message(chat, *message_id).await?
        }
        tasks::CmdType::Url => return Err(Error::new("invalid cmd type")),
    };

    let media = message
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let downloadable = Arc::new(Downloadable::Media(media));

    let mut work_handles = VecDeque::new();

    let total_chunks_num = if total_length > MAX_CHUNK_SIZE as u64 {
        (total_length as f32 / MAX_CHUNK_SIZE as f32).ceil() as i32
    } else {
        1
    };
    let mut current_chunk_num = 0;

    while current_chunk_num < total_chunks_num {
        let telegram_user_clone = telegram_user.clone();
        let downloadable_clone = downloadable.clone();

        work_handles.push_back(tokio::spawn(async move {
            let download = telegram_user_clone.iter_download(downloadable_clone.as_ref());

            download
                .skip_chunks(current_chunk_num)
                .next()
                .await
                .map_err(|e| {
                    Error::new_telegram_invocation(
                        e,
                        "failed to get next chunk from tg file downloader",
                    )
                })
        }));

        current_chunk_num += 1;

        if current_chunk_num % WORKER_COUNT == 0 || current_chunk_num == total_chunks_num {
            let mut chunk = Vec::new();

            while let Some(handle) = work_handles.pop_front() {
                let mut chunk_part = handle
                    .await
                    .map_err(|e| Error::new(format!("failed to join handle: {}", e)))??
                    .ok_or_else(|| Error::new("failed to get chunk from tg file downloader"))?;

                chunk.append(&mut chunk_part);
            }

            upload_response = upload_file(
                &upload_session,
                &chunk,
                current_length,
                total_length,
                &http_client,
            )
            .await?;

            current_length += chunk.len() as u64;
            progress
                .set_current_length(id.to_owned(), current_length)
                .await?;
        }
    }

    let filename = upload_response
        .ok_or_else(|| Error::new("failed to get drive item after upload"))?
        .name
        .ok_or_else(|| Error::new("drive item name not found"))?;

    if message_id_forward.is_some() {
        message.delete().await?;
    }

    Ok(filename)
}

#[add_context]
#[add_trace]
async fn upload_file(
    upload_session: &UploadSession,
    buffer: &[u8],
    current_length: u64,
    total_length: u64,
    http_client: &reqwest::Client,
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
                    // normal
                    // 408: Request Timeout
                    // 500: Internal Server Error
                    // 502: Bad Gateway
                    // 503: Service Unavailable
                    // 504: Gateway Timeout
                    // 416: Requested Range Not Satisfiable, probably because the fragment has already been received
                    //
                    // probably has some problem
                    // 409: Conflict, probably caused by rename, too many files with the same name uploaded at once
                    // 404: Not Found, probably because the item has already been uploaded

                    if status_code.as_u16() == 416 {
                        break;
                    }
                }

                if tries < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    continue;
                }

                return Err(Error::new_onedrive(e, "failed to upload part"));
            }
        }
    }

    Ok(upload_response)
}
