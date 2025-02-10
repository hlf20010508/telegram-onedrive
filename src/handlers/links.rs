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
    message::{MessageInfo, TelegramMessage},
    state::AppState,
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
    } else {
        return Err(anyhow!(format_unknown_command_help(PATTERN)));
    }

    Ok(())
}
