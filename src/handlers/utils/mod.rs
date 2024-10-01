/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub mod message;
pub mod text;
pub mod upload;
pub mod zip;

use crate::{
    client::onedrive::invalid_name::{INVALID_COMPONENT, INVALID_NAME, INVALID_NAME_PREFIX},
    error::{Error, Result, ResultExt},
    message::TelegramMessage,
    utils::{get_current_timestamp, get_ext},
};
use grammers_client::types::media::{Document, Media};
use mime_guess::get_mime_extensions_str;
use percent_encoding::percent_decode_str;
use proc_macros::{add_context, add_trace};
use regex::Regex;
use reqwest::{header, Response, StatusCode};
use std::collections::HashMap;
use url::Url;

#[add_context]
#[add_trace]
pub fn get_filename(url: &str, response: &Response) -> Result<String> {
    if response.status() != StatusCode::OK {
        return Err(Error::new("file from url not found"));
    }

    let filename = match get_filename_from_cd(response)? {
        Some(filename) => Some(filename),
        None => get_filename_from_url(url)?,
    };

    let content_type = match response.headers().get(header::CONTENT_TYPE) {
        Some(content_type) => content_type
            .to_str()
            .map_err(|e| Error::new("header Content-Type has invisible ASCII chars").raw(e))?,
        None => "application/octet-stream",
    };

    let exts = guess_exts(content_type);

    let filename = match filename {
        Some(filename) => {
            let mut filename = filename;

            if !exts.is_empty() && content_type != "application/octet-stream" {
                let origin_ext = get_ext(&filename);

                if filename.len() < 100 {
                    if !exts.contains(&origin_ext) {
                        filename = filename + "." + &exts[0];
                    }
                } else {
                    let timestamp = get_current_timestamp().to_string();

                    if exts.contains(&origin_ext) {
                        filename = timestamp + "." + &origin_ext;
                    } else {
                        filename = filename + "." + &exts[0];
                    }
                }
            } else if filename.len() > 100 {
                filename = get_current_timestamp().to_string();
            }

            filename
        }
        None => {
            let mut filename = get_current_timestamp().to_string();

            if let Some(ext) = exts.first() {
                if content_type != "application/octet-stream" {
                    filename = filename + "." + ext;
                }
            }

            filename
        }
    };

    Ok(preprocess_url_file_name(&filename))
}

#[add_context]
#[add_trace]
fn get_filename_from_cd(response: &Response) -> Result<Option<String>> {
    if let Some(cd) = response.headers().get(header::CONTENT_DISPOSITION) {
        let cd = cd.to_str().map_err(|e| {
            Error::new("header Content-Disposition has invisible ASCII chars").raw(e)
        })?;

        let pattern = r"filename=(.+)";
        let re = Regex::new(pattern)
            .map_err(|e| Error::new("invalid regex pattern").raw(e).details(pattern))
            .unwrap_or_trace();

        let filename = re
            .captures(cd)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));

        if let Some(filename) = filename {
            if !filename.is_empty() {
                let filename = filename
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string();

                tracing::debug!("got url filename from Content-Disposition: {}", filename);

                return Ok(Some(filename));
            }
        }
    }

    Ok(None)
}

#[add_context]
#[add_trace]
fn get_filename_from_url(url: &str) -> Result<Option<String>> {
    let parsed_url = Url::parse(url).map_err(|e| Error::new("failed to parse url").raw(e))?;
    let captured_value_dict = parsed_url
        .query_pairs()
        .into_iter()
        .map(|q| (q.0.to_string(), q.1.to_string().to_lowercase()))
        .collect::<HashMap<String, String>>();

    let file_param_name_list = ["name", "filename", "file_name", "title", "file"];

    let filename = {
        let mut filename = None;

        for item_name in captured_value_dict.keys() {
            if file_param_name_list.contains(&item_name.as_str()) {
                filename = Some(captured_value_dict[item_name].clone());
                break;
            }
        }

        filename
    };

    let filename = match filename {
        Some(filename) => filename,
        None => {
            let last_segment = parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("");

            percent_decode_str(last_segment)
                .decode_utf8_lossy()
                .to_string()
        }
    };

    if filename.is_empty() {
        Ok(None)
    } else {
        tracing::debug!("got url filename from url: {}", filename);

        Ok(Some(filename))
    }
}

#[add_trace]
fn guess_exts(content_type: &str) -> Vec<String> {
    let content_type = {
        // text/html
        let mut content_type = content_type.trim().to_string();

        // text/html; charset=utf-8
        let pattern = r"([^;]+)";
        let re = Regex::new(pattern)
            .map_err(|e| Error::new("invalid regex pattern").raw(e).details(pattern))
            .unwrap_or_trace();

        if let Some(cap) = re.captures(&content_type) {
            if let Some(mime_type) = cap.get(1) {
                content_type = mime_type.as_str().trim().to_string();
            }
        }

        content_type
    };

    match get_mime_extensions_str(&content_type) {
        Some(exts) => exts
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        None => Vec::new(),
    }
}

#[add_trace]
fn validate_filename(filename: &str) -> bool {
    if filename.is_empty() || INVALID_NAME.contains(&filename) {
        return false;
    }

    for component in INVALID_COMPONENT {
        if filename.contains(component) {
            return false;
        }
    }

    true
}

#[add_context]
#[add_trace]
pub async fn is_root_path_valid(root_path: &str, message: TelegramMessage) -> Result<bool> {
    if root_path.starts_with('/') {
        Ok(true)
    } else {
        let response = "directory path should start with /";
        message.reply(response).await.details(response)?;

        Ok(false)
    }
}

#[add_trace]
fn preprocess_url_file_name(filename: &str) -> String {
    if validate_filename(filename) {
        filename
            .trim()
            .trim_start_matches(INVALID_NAME_PREFIX)
            .to_string()
    } else {
        let sp = filename
            .split('.')
            .map(|spi| spi.to_string())
            .collect::<Vec<String>>();

        if sp.len() > 1 {
            let ext = sp.last().unwrap();

            get_current_timestamp().to_string() + "." + ext
        } else {
            get_current_timestamp().to_string()
        }
    }
}

#[add_trace]
pub fn preprocess_tg_file_name(media: &Media) -> String {
    let (filename, id) = match media {
        Media::Photo(file) => return file.id().to_string() + ".jpg",
        Media::Document(file) => get_tg_document_name_and_id(file),
        Media::Sticker(file) => get_tg_document_name_and_id(&file.document),
        _ => Default::default(),
    };

    if validate_filename(&filename) {
        filename
            .trim()
            .trim_start_matches(INVALID_NAME_PREFIX)
            .to_string()
    } else {
        let ext = get_ext(&filename);

        id.to_string() + "." + &ext
    }
}

#[add_trace]
fn get_tg_document_name_and_id(document: &Document) -> (String, i64) {
    let mut filename = document.name().to_string();
    let file_id = document.id();
    if filename.is_empty() {
        if let Some(mime) = document.mime_type() {
            let exts = guess_exts(mime);

            if exts.is_empty() {
                filename = file_id.to_string();
            } else {
                filename = file_id.to_string() + "." + &exts[0];
            }
        }
    }

    (filename, file_id)
}

#[add_trace]
pub fn get_tg_file_size(media: &Media) -> u64 {
    let size = match media {
        Media::Photo(file) => file.size(),
        Media::Document(file) => file.size(),
        Media::Sticker(file) => file.document.size(),
        _ => Default::default(),
    };

    size as u64
}