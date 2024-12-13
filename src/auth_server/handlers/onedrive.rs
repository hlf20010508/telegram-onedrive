/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::models::CodeParams;
use crate::{auth_server::SenderOD, error::HttpError};
use axum::{debug_handler, extract::Query, response::Result, Extension};

pub const CODE_PATH: &str = "/auth";

#[debug_handler]
pub async fn code_handler(
    Extension(SenderOD(tx)): Extension<SenderOD>,
    Query(CodeParams { code }): Query<CodeParams>,
) -> Result<String> {
    tracing::debug!("received od auth code: {}", code);

    tx.send(code).await.map_err(HttpError::new)?;

    Ok("Authorization successful!".to_string())
}
