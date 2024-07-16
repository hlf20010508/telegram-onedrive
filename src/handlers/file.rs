/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::client::files::DownloadIter;
use grammers_client::grammers_tl_types::enums::{InputFileLocation, MessageFwdHeader};
use grammers_client::{Client, InputMessage};
use std::io::Cursor;
// use grammers_client::grammers_tl_types::enums::{
//     InputFileLocation, MessageFwdHeader, PhotoSize as PhotoSizeRaw,
// };
use grammers_client::types::{Chat, Media, Message};
use std::sync::Arc;

use crate::client::ext::TelegramExt;
use crate::error::{Error, Result};
use crate::handlers::utils::{get_tg_file_size, preprocess_tg_file_name};
use crate::state::AppState;
use crate::tasker::CmdType;
use crate::{check_in_group, check_od_login, check_senders, check_tg_login};

pub async fn handler(message: Arc<Message>, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_tg_login!(message, state);
    check_od_login!(message, state);

    let telegram_user = &state.telegram_user;
    let onedrive = &state.onedrive;
    let task_session = state.task_session.clone();

    let chat_user = telegram_user
        .client
        .get_chat(message.clone())
        .await?
        .ok_or_else(|| Error::new("failed to get user chat from message"))?;

    let message_user = telegram_user
        .client
        .get_message(&chat_user, message.id())
        .await?;

    let media = message_user
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let filename = preprocess_tg_file_name(&media);

    let total_length = get_tg_file_size(&media);

    let root_path = onedrive.get_root_path(true).await?;

    let (upload_session, upload_session_meta) = onedrive
        .multipart_upload_session_builder(&root_path, &filename)
        .await?;

    let current_length = {
        match upload_session_meta.next_expected_ranges.get(0) {
            Some(range) => range.start,
            None => 0,
        }
    };

    let mut message_id = message.id();
    let chat_bot_hex = message.chat().pack().to_hex();
    let chat_user_hex = chat_user.pack().to_hex();

    let cmd_type = match media {
        Media::Photo(_) => CmdType::Photo,
        Media::Document(_) | Media::Sticker(_) => CmdType::File,
        _ => Err(Error::new(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    if let Some(forward_header) = message_user.forward_header() {
        match forward_header {
            MessageFwdHeader::Header(_) => match media {
                Media::Photo(file) => {
                    if let InputFileLocation::InputPhotoFileLocation(mut location) =
                        file.to_raw_input_location().unwrap()
                    {
                        location.thumb_size = "m".to_string();

                        let input_file_location =
                            InputFileLocation::InputPhotoFileLocation(location);

                        message_id = upload_thumb(
                            &telegram_user.client,
                            &chat_user,
                            &filename,
                            input_file_location,
                        )
                        .await?;
                    }
                }
                Media::Document(file) => {
                    if let InputFileLocation::InputDocumentFileLocation(mut location) =
                        file.to_raw_input_location().unwrap()
                    {
                        location.thumb_size = "m".to_string();

                        let input_file_location =
                            InputFileLocation::InputDocumentFileLocation(location);

                        message_id = upload_thumb(
                            &telegram_user.client,
                            &chat_user,
                            &filename,
                            input_file_location,
                        )
                        .await?;
                    }
                }
                Media::Sticker(file) => {
                    if let InputFileLocation::InputDocumentFileLocation(mut location) =
                        file.document.to_raw_input_location().unwrap()
                    {
                        location.thumb_size = "m".to_string();

                        let input_file_location =
                            InputFileLocation::InputDocumentFileLocation(location);

                        message_id = upload_thumb(
                            &telegram_user.client,
                            &chat_user,
                            &filename,
                            input_file_location,
                        )
                        .await?;
                    }
                }
                _ => Err(Error::new(
                    "media type is not one of photo, document and sticker",
                ))?,
            },
        }
    }

    task_session
        .insert_task(
            cmd_type,
            &filename,
            &root_path,
            None,
            upload_session.upload_url(),
            current_length,
            total_length,
            &chat_bot_hex,
            &chat_user_hex,
            message_id,
            Some(message.id()),
        )
        .await?;

    Ok(())
}

async fn upload_thumb(
    client: &Client,
    chat: &Chat,
    filename: &str,
    input_file_location: InputFileLocation,
) -> Result<i32> {
    let mut download = DownloadIter::new_from_location(client, input_file_location);

    let mut buffer = Vec::new();

    while let Some(chunk) = download
        .next()
        .await
        .map_err(|e| Error::context(e, "failed to download thumb chunk"))?
    {
        buffer.extend_from_slice(&chunk);
    }

    let size = buffer.len();
    let mut stream = Cursor::new(buffer);
    let uploaded = client
        .upload_stream(&mut stream, size, "thumb.jpg".to_string())
        .await
        .map_err(|e| Error::context(e, "failed to upload thumb"))?;

    let message_id = client
        .send_message(
            chat,
            InputMessage::text(&format!("\u{200B}{}", filename)).photo(uploaded),
        )
        .await
        .map_err(|e| Error::context(e, "failed to send message for forwarded"))?
        .id();

    Ok(message_id)
}

// trait PhotoSizeRawExt {
//     fn size(&self) -> usize;
//     fn download_to_memory(&self) -> Vec<u8>;
// }

// impl PhotoSizeRawExt for PhotoSizeRaw {
//     fn size(&self) -> usize {
//         match self {
//             PhotoSizeRaw::Empty(_) => 0,
//             PhotoSizeRaw::Size(size) => size.size as usize,
//             PhotoSizeRaw::PhotoCachedSize(size) => size.bytes.len(),
//             PhotoSizeRaw::PhotoStrippedSize(size) => {
//                 let bytes = &size.bytes;
//                 if bytes.len() < 3 || bytes[0] != 0x01 {
//                     return 0;
//                 }
//                 size.bytes.len() + 622
//             }
//             PhotoSizeRaw::Progressive(size) => size.sizes.iter().sum::<i32>() as usize,
//             PhotoSizeRaw::PhotoPathSize(size) => size.bytes.len(),
//         }
//     }
//     fn download_to_memory(&self) -> Vec<u8> {
//         match self {
//             PhotoSizeRaw::Size(size) => todo!(),
//             PhotoSizeRaw::PhotoCachedSize(size) => size.bytes.clone(),
//             PhotoSizeRaw::PhotoStrippedSize(size) => {
//                 // Based on https://core.tlgr.org/api/files#stripped-thumbnails
//                 let bytes = &size.bytes;
//                 if bytes.len() < 3 || bytes[0] != 0x01 {
//                     return Vec::new();
//                 }

//                 let header = vec![
//                     0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01,
//                     0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x28,
//                     0x1c, 0x1e, 0x23, 0x1e, 0x19, 0x28, 0x23, 0x21, 0x23, 0x2d, 0x2b, 0x28, 0x30,
//                     0x3c, 0x64, 0x41, 0x3c, 0x37, 0x37, 0x3c, 0x7b, 0x58, 0x5d, 0x49, 0x64, 0x91,
//                     0x80, 0x99, 0x96, 0x8f, 0x80, 0x8c, 0x8a, 0xa0, 0xb4, 0xe6, 0xc3, 0xa0, 0xaa,
//                     0xda, 0xad, 0x8a, 0x8c, 0xc8, 0xff, 0xcb, 0xda, 0xee, 0xf5, 0xff, 0xff, 0xff,
//                     0x9b, 0xc1, 0xff, 0xff, 0xff, 0xfa, 0xff, 0xe6, 0xfd, 0xff, 0xf8, 0xff, 0xdb,
//                     0x00, 0x43, 0x01, 0x2b, 0x2d, 0x2d, 0x3c, 0x35, 0x3c, 0x76, 0x41, 0x41, 0x76,
//                     0xf8, 0xa5, 0x8c, 0xa5, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8,
//                     0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8,
//                     0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8,
//                     0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8,
//                     0xf8, 0xf8, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01,
//                     0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01, 0xff, 0xc4, 0x00, 0x1f, 0x00,
//                     0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
//                     0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
//                     0x0a, 0x0b, 0xff, 0xc4, 0x00, 0xb5, 0x10, 0x00, 0x02, 0x01, 0x03, 0x03, 0x02,
//                     0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7d, 0x01, 0x02, 0x03,
//                     0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
//                     0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15,
//                     0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18,
//                     0x19, 0x1a, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38,
//                     0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55,
//                     0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a,
//                     0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87,
//                     0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2,
//                     0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6,
//                     0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca,
//                     0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2, 0xe3, 0xe4,
//                     0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
//                     0xf8, 0xf9, 0xfa, 0xff, 0xc4, 0x00, 0x1f, 0x01, 0x00, 0x03, 0x01, 0x01, 0x01,
//                     0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
//                     0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0xff, 0xc4, 0x00,
//                     0xb5, 0x11, 0x00, 0x02, 0x01, 0x02, 0x04, 0x04, 0x03, 0x04, 0x07, 0x05, 0x04,
//                     0x04, 0x00, 0x01, 0x02, 0x77, 0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21,
//                     0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71, 0x13, 0x22, 0x32, 0x81, 0x08,
//                     0x14, 0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0, 0x15, 0x62,
//                     0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19, 0x1a,
//                     0x26, 0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44,
//                     0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
//                     0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76,
//                     0x77, 0x78, 0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a,
//                     0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5,
//                     0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9,
//                     0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4,
//                     0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8,
//                     0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xff, 0xda,
//                     0x00, 0x0c, 0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3f, 0x00,
//                 ];
//                 let mut footer = vec![0xff, 0xd9];
//                 let mut real = header;
//                 real[164] = bytes[1];
//                 real[166] = bytes[2];

//                 let mut bytes_clone = bytes.clone()[3..].to_vec();
//                 real.append(&mut bytes_clone);
//                 real.append(&mut footer);

//                 real
//             }
//             PhotoSizeRaw::PhotoPathSize(size) => {
//                 // Based on https://core.tlgr.org/api/files#vector-thumbnails
//                 let lookup = "AACAAAAHAAALMAAAQASTAVAAAZaacaaaahaaalmaaaqastava.az0123456789-,";
//                 let mut path = String::from("M");
//                 for num in &size.bytes {
//                     let num = *num;
//                     if num >= 128 + 64 {
//                         path.push(lookup.chars().nth((num - 128 - 64) as usize).unwrap());
//                     } else {
//                         if num >= 128 {
//                             path.push(',');
//                         } else if num >= 64 {
//                             path.push('-');
//                         }
//                         path.push((num & 63) as char);
//                     }
//                 }
//                 path.push('z');
//                 let res = format!(
//                     r#"<?xml version="1.0" encoding="utf-8"?>
//                     <svg
//                         version="1.1"
//                         xmlns="http://www.w3.org/2000/svg"
//                         xmlns:xlink="http://www.w3.org/1999/xlink"
//                         viewBox="0 0 512 512"
//                         xml:space="preserve"
//                     >
//                         <path d="{path}"/>
//                     </svg>"#
//                 );

//                 res.as_bytes().to_vec()
//             }
//             _ => Vec::new(),
//         }
//     }
// }
