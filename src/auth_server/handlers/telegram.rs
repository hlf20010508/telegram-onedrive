/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{models::CodeParams, TG_CODE_EVENT};
use crate::error::{Error, Result};
use axum::{
    debug_handler,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension, Json,
};
use proc_macros::{add_context, add_trace};
use socketioxide::SocketIo;
use std::sync::Arc;
use tokio::fs;

pub const INDEX_PATH: &str = "/";

#[debug_handler]
#[add_context]
#[add_trace]
pub async fn index_handler() -> Result<Html<String>> {
    let html = fs::read_to_string("./index.html")
        .await
        .map_err(|e| Error::new("failed to read index.html").raw(e))?;

    Ok(Html(html))
}

pub const CODE_PATH: &str = "/tg";

#[debug_handler]
#[add_context]
#[add_trace]
pub async fn code_handler(
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(CodeParams { code }): Json<CodeParams>,
) -> Result<Response> {
    tracing::debug!("received tg auth code: {}", code);

    socketio
        .emit(TG_CODE_EVENT, code)
        .map_err(|e| Error::new("failed to emit tg_code").raw(e))?;

    Ok(StatusCode::OK.into_response())
}
