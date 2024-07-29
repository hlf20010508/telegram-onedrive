/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use futures::FutureExt;
use native_tls::TlsConnector;
use proc_macros::{add_context, add_trace};
use rust_socketio::asynchronous::{
    Client as SocketIoClient, ClientBuilder as SocketIoClientBuilder,
};
use rust_socketio::Payload;
use tokio::sync::mpsc;

use crate::error::{Error, Result};

#[add_context]
#[add_trace]
pub async fn socketio_client(
    event: &str,
    port: u16,
    use_reverse_proxy: bool,
) -> Result<(SocketIoClient, mpsc::Receiver<String>)> {
    let (tx, rx) = mpsc::channel(1);

    let tls_connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|e| Error::new_tls(e, "failed to create tls connector for socketio client"))?;

    let protocol = if use_reverse_proxy { "http" } else { "https" };

    let socketio_client = SocketIoClientBuilder::new(format!("{}://127.0.0.1:{}/", protocol, port))
        .tls_config(tls_connector)
        .on(event, move |payload, _socket| {
            let tx: mpsc::Sender<String> = tx.clone();
            async move {
                if let Payload::Text(values) = payload {
                    if let Some(value) = values.first() {
                        let code = serde_json::from_value::<String>(value.to_owned()).unwrap();

                        tx.send(code).await.unwrap();
                    }
                }
            }
            .boxed()
        })
        .connect()
        .await
        .map_err(|e| Error::new_socket_io_client(e, "failed to connect to auth server"))?;

    Ok((socketio_client, rx))
}

#[add_context]
#[add_trace]
pub async fn socketio_disconnect(socketio_client: SocketIoClient) -> Result<()> {
    socketio_client
        .disconnect()
        .await
        .map_err(|e| Error::new_socket_io_client(e, "failed to disconnect from auth server"))?;

    Ok(())
}
