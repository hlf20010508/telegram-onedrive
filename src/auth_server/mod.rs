/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod auto_abort;
mod handlers;
mod var;

use axum::routing::{get, post};
use axum::{Extension, Router};
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

pub use var::{SERVER_PORT, TG_CODE_EVENT};

use auto_abort::AutoAbortHandle;
use handlers::telegram;

use crate::error::{Error, Result};

pub async fn spawn() -> Result<AutoAbortHandle> {
    let (socketio_layer, socketio) = SocketIo::new_layer();

    socketio.ns("/", |_s: SocketRef| {});

    let router = Router::new()
        .route(telegram::INDEX_PATH, get(telegram::index_handler))
        .route(telegram::CODE_PATH, post(telegram::code_handler))
        .layer(socketio_layer)
        .layer(Extension(Arc::new(socketio)));

    let server = TcpListener::bind(format!("127.0.0.1:{}", SERVER_PORT))
        .await
        .map_err(|e| Error::context(e, "failed to create tcp listener"))?;

    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = Arc::clone(&shutdown_flag);

    let abort_handle = tokio::spawn(async move {
        axum::serve(server, router)
            .with_graceful_shutdown(async move {
                while !shutdown_flag_clone.load(Ordering::SeqCst) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            })
            .await
            .unwrap();
    })
    .abort_handle();

    let auto_abort_handle = AutoAbortHandle::new(abort_handle, shutdown_flag);

    Ok(auto_abort_handle)
}
