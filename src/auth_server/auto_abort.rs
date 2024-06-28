/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::task::AbortHandle;

pub struct AutoAbortHandle {
    handle: AbortHandle,
    shutdown_flag: Arc<AtomicBool>,
}

impl AutoAbortHandle {
    pub fn new(handle: AbortHandle, shutdown_flag: Arc<AtomicBool>) -> Self {
        Self {
            handle,
            shutdown_flag,
        }
    }
}

impl Drop for AutoAbortHandle {
    fn drop(&mut self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        self.handle.abort();
    }
}
