/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::extract::Query;
use axum::{debug_handler, Extension};
use proc_macros::add_trace;
use socketioxide::SocketIo;
use std::sync::Arc;

use super::models::CodeParams;
use super::OD_CODE_EVENT;
use crate::error::{Error, Result};

pub const CODE_PATH: &str = "/auth";

#[debug_handler]
#[add_trace(context)]
pub async fn code_handler(
    Extension(socketio): Extension<Arc<SocketIo>>,
    Query(CodeParams { code }): Query<CodeParams>,
) -> Result<String> {
    socketio
        .emit(OD_CODE_EVENT, code)
        .map_err(|e| Error::new_socket_io_server_broadcast(e, "failed to emit od_code"))?;

    tracing::debug!("od code emitted");

    Ok("Authorization successful!".to_string())
}
