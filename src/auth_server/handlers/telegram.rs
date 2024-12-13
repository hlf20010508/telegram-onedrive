/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::models::CodeParams;
use crate::{auth_server::SenderTG, error::HttpError};
use anyhow::Context;
use axum::{
    debug_handler,
    http::StatusCode,
    response::{Html, IntoResponse, Response, Result},
    Extension, Json,
};
use tokio::fs;

pub const INDEX_PATH: &str = "/";

#[debug_handler]
pub async fn index_handler() -> Result<Html<String>> {
    let html = fs::read_to_string("./index.html")
        .await
        .context("failed to read index.html")
        .map_err(|e| HttpError::new(format!("{:#}", e)))?;

    Ok(Html(html))
}

pub const CODE_PATH: &str = "/tg";

#[debug_handler]
pub async fn code_handler(
    Extension(SenderTG(tx)): Extension<SenderTG>,
    Json(CodeParams { code }): Json<CodeParams>,
) -> Result<Response> {
    tracing::debug!("received tg auth code: {}", code);

    tx.send(code)
        .await
        .context("failed to send tg auth code")
        .map_err(|e| HttpError::new(format!("{:#}", e)))?;

    Ok(StatusCode::OK.into_response())
}
