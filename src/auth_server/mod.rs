/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod auto_abort;
mod cert;
mod handlers;

use crate::{
    env::{ENV, Env},
    error::ResultExt,
};
use anyhow::{Context, Result};
use auto_abort::AutoAbortHandle;
use axum::{
    Extension, Router,
    routing::{get, post},
};
use axum_server::Handle;
use cert::get_rustls_config;
use handlers::{onedrive, telegram};
use std::net::TcpListener;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone)]
struct SenderTG(Sender<String>);

#[derive(Clone)]
struct SenderOD(Sender<String>);

pub async fn spawn() -> Result<(Receiver<String>, Receiver<String>, AutoAbortHandle)> {
    tracing::debug!("spawning auth server");

    let Env {
        port,
        use_reverse_proxy,
        ..
    } = ENV.get().unwrap();

    let (tx_tg, rx_tg) = mpsc::channel(1);
    let (tx_od, rx_od) = mpsc::channel(1);

    let router = Router::new()
        .route(telegram::INDEX_PATH, get(telegram::index_handler))
        .route(telegram::CODE_PATH, post(telegram::code_handler))
        .route(onedrive::CODE_PATH, get(onedrive::code_handler))
        .layer(Extension(SenderTG(tx_tg)))
        .layer(Extension(SenderOD(tx_od)));

    let server =
        TcpListener::bind(format!("0.0.0.0:{}", port)).context("failed to create tcp listener")?;

    let shutdown_handle = Handle::new();
    let shutdown_handle_clone = shutdown_handle.clone();

    let abort_handle = if use_reverse_proxy.to_owned() {
        tracing::info!("auth server listening on http://0.0.0.0:{}", port);

        tokio::spawn(async move {
            axum_server::from_tcp(server)
                .handle(shutdown_handle_clone)
                .serve(router.into_make_service())
                .await
                .context("auth server failed to serve")
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
                .context("auth server failed to serve")
                .trace();
        })
        .abort_handle()
    };

    let auto_abort_handle = AutoAbortHandle::new(abort_handle, shutdown_handle);

    Ok((rx_tg, rx_od, auto_abort_handle))
}
