/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod auto_abort;
mod cert;
mod handlers;
mod var;

use axum::routing::{get, post};
use axum::{Extension, Router};
use axum_server::Handle;
use proc_macros::{add_context, add_trace};
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use std::net::TcpListener;
use std::sync::Arc;

pub use var::{OD_CODE_EVENT, TG_CODE_EVENT};

use auto_abort::AutoAbortHandle;
use cert::get_rustls_config;
use handlers::{onedrive, telegram};

use crate::env::Env;
use crate::error::{Error, Result};

#[add_context]
#[add_trace]
pub async fn spawn(
    Env {
        port,
        use_reverse_proxy,
        ..
    }: &Env,
) -> Result<AutoAbortHandle> {
    tracing::debug!("spawning auth server");

    let (socketio_layer, socketio) = SocketIo::new_layer();

    socketio.ns("/", |_s: SocketRef| {});

    let router = Router::new()
        .route(telegram::INDEX_PATH, get(telegram::index_handler))
        .route(telegram::CODE_PATH, post(telegram::code_handler))
        .route(onedrive::CODE_PATH, get(onedrive::code_handler))
        .layer(socketio_layer)
        .layer(Extension(Arc::new(socketio)));

    let server = TcpListener::bind(format!("0.0.0.0:{}", port))
        .map_err(|e| Error::new("failed to create tcp listener").raw(e))?;

    let shutdown_handle = Handle::new();
    let shutdown_handle_clone = shutdown_handle.clone();

    let abort_handle = if use_reverse_proxy.to_owned() {
        tracing::info!("auth server listening on http://0.0.0.0:{}", port);

        tokio::spawn(async move {
            axum_server::from_tcp(server)
                .handle(shutdown_handle_clone)
                .serve(router.into_make_service())
                .await
                .map_err(|e| Error::new("auth server failed to serve").raw(e))
                .trace();
        })
        .abort_handle()
    } else {
        let config = get_rustls_config().await?;

        tracing::info!("auth server listening on https://0.0.0.0:{}", port);

        tokio::spawn(async move {
            axum_server::from_tcp_rustls(server, config)
                .handle(shutdown_handle_clone)
                .serve(router.into_make_service())
                .await
                .map_err(|e| Error::new("auth server failed to serve").raw(e))
                .trace();
        })
        .abort_handle()
    };

    let auto_abort_handle = AutoAbortHandle::new(abort_handle, shutdown_handle);

    Ok(auto_abort_handle)
}
