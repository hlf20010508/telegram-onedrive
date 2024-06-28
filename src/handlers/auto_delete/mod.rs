/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use grammers_client::types::Message;

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::{check_in_group, check_senders};

pub const PATTERN: &str = "/autoDelete";

pub async fn handler(message: Message, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);

    let mut should_auto_delete = state.should_auto_delete.lock().await;

    *should_auto_delete = !*should_auto_delete;

    if *should_auto_delete {
        message
            .respond(docs::WILL_AUTO_DELETE)
            .await
            .map_err(|e| Error::context(e, "failed to respond message in auto_delete"))?;
    } else {
        message
            .respond(docs::WONT_AUTO_DELETE)
            .await
            .map_err(|e| Error::context(e, "failed to respond message in auto_delete"))?;
    }

    Ok(())
}
