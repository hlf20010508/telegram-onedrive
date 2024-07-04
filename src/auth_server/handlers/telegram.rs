/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::{debug_handler, Extension, Json};
use socketioxide::SocketIo;
use std::sync::Arc;
use tokio::fs;

use super::models::CodeParams;
use super::TG_CODE_EVENT;
use crate::error::{Error, Result};

pub const INDEX_PATH: &str = "/";

#[debug_handler]
pub async fn index_handler() -> Result<Html<String>> {
    let html = fs::read_to_string("./index.html")
        .await
        .map_err(|e| Error::context(e, "failed to read index.html"))?;

    Ok(Html(html))
}

pub const CODE_PATH: &str = "/tg";

#[debug_handler]
pub async fn code_handler(
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(CodeParams { code }): Json<CodeParams>,
) -> Result<Response> {
    socketio
        .emit(TG_CODE_EVENT, code)
        .map_err(|e| Error::context(e, "failed to emit tg_code"))?;

    tracing::debug!("tg code emitted");

    Ok(StatusCode::OK.into_response())
}