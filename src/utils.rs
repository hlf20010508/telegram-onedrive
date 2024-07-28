/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use chrono::Utc;
use proc_macros::add_trace;
use reqwest::header;

use crate::error::{Error, Result};

#[add_trace]
pub fn get_current_timestamp() -> i64 {
    Utc::now().timestamp()
}

#[add_trace(context)]
pub async fn get_http_client() -> Result<reqwest::Client> {
    const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";

    let headers = {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            USER_AGENT
                .parse()
                .map_err(|e| Error::new_http_header_value(e, "failed to parse user agent"))?,
        );

        headers
    };

    reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|e| Error::new_http_request(e, "failed to build http client"))
}

#[add_trace]
pub fn get_ext(filename: &str) -> String {
    filename.split('.').last().unwrap().to_lowercase()
}
