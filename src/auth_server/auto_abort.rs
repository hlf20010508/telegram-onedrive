/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum_server::Handle;
use tokio::task::AbortHandle;

pub struct AutoAbortHandle {
    abort_handle: AbortHandle,
    shutdown_handle: Handle,
}

impl AutoAbortHandle {
    pub const fn new(abort_handle: AbortHandle, shutdown_handle: Handle) -> Self {
        Self {
            abort_handle,
            shutdown_handle,
        }
    }
}

impl Drop for AutoAbortHandle {
    fn drop(&mut self) {
        self.shutdown_handle.shutdown();
        self.abort_handle.abort();

        tracing::debug!("auth server auto aborted");
    }
}
