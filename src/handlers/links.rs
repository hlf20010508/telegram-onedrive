/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    docs::{format_help, format_unknown_command_help},
    link,
    utils::{
        message::{get_message_info, get_message_link},
        text::cmd_parser,
    },
};
use crate::{
    error::ResultExt,
    message::{ChatEntity, MessageInfo, TelegramMessage},
    state::AppState,
    tasker::BatchAborter,
};
use anyhow::{anyhow, Context, Result};
use grammers_client::InputMessage;
use proc_macros::{check_in_group, check_od_login, check_senders, check_tg_login};

pub const PATTERN: &str = "/links";

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let cmd = cmd_parser(message.text());

    if cmd.len() == 2 && cmd[1] == "help" {
        // /links help
        message
            .respond(InputMessage::html(format_help(PATTERN)))
            .await
            .context("help")?;
    } else if cmd.len() == 3 {
        // /links $message_link $num
        let link_head = &cmd[1];
        let link_num = cmd[2]
            .parse::<usize>()
            .context("failed to parse link number")?;

        let MessageInfo {
            chat_entity,
            id: head_message_id,
        } = get_message_info(link_head)?;

        let telegram_user = &state.telegram_user;

        let chat_user = telegram_user
            .get_chat(&ChatEntity::from(message.chat()))
            .await?;

        let mut batch_aborters = state.task_session.batch_aborters.lock().await;
        // /links may be in a batch
        #[allow(clippy::option_if_let_else)]
        let (cancellation_token, wrapped_in_batch) =
            if let Some(batch_aborter) = batch_aborters.get(&(chat_user.id(), message.id())) {
                (batch_aborter.token.clone(), true)
            } else {
                let batch_aborter = BatchAborter::new();
                let cancellation_token = batch_aborter.token.clone();
                batch_aborters.insert((chat_user.id(), message.id()), batch_aborter);

                (cancellation_token, false)
            };
        // allow cancellation
        drop(batch_aborters);

        let fut = async {
            for offset in 0..link_num {
                let message_origin_id = head_message_id + offset as i32;
                let message_link = get_message_link(&chat_entity, message_origin_id);

                let mut message_clone = message.clone();
                message_clone.override_text(message_link.clone());

                if link::handler(message_clone, state.clone()).await.is_err() {
                    message
                        .reply(format!("message {} not found", message_link))
                        .await
                        .unwrap_or_trace();

                    continue;
                }
            }
        };

        tokio::select! {
            () = fut => {}
            () = cancellation_token.cancelled() => {}
        }

        if !wrapped_in_batch {
            let mut batch_aborters = state.task_session.batch_aborters.lock().await;
            if let Some(batch_aborter) = batch_aborters.get_mut(&(chat_user.id(), message.id())) {
                batch_aborter.processing = false;
            }
        }
    } else {
        return Err(anyhow!(format_unknown_command_help(PATTERN)));
    }

    Ok(())
}
