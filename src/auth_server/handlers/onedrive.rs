/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{models::CodeParams, OD_CODE_EVENT};
use crate::error::{Error, Result};
use axum::{debug_handler, extract::Query, Extension};
use proc_macros::{add_context, add_trace};
use socketioxide::SocketIo;
use std::sync::Arc;

pub const CODE_PATH: &str = "/auth";

#[debug_handler]
#[add_context]
#[add_trace]
pub async fn code_handler(
    Extension(socketio): Extension<Arc<SocketIo>>,
    Query(CodeParams { code }): Query<CodeParams>,
) -> Result<String> {
    tracing::debug!("received od auth code: {}", code);

    socketio
        .emit(OD_CODE_EVENT, code)
        .map_err(|e| Error::new("failed to emit od_code").raw(e))?;

    Ok("Authorization successful!".to_string())
}
