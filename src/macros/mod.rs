/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub mod docs;

#[macro_export]
macro_rules! check_in_group {
    ($message:ident) => {
        match $message.chat() {
            grammers_client::types::Chat::Group(_) => {}
            _ => {
                $message
                    .respond(crate::macros::docs::CHECK_IN_GROUP_FAILED)
                    .await
                    .map_err(|e| {
                        crate::error::Error::context(
                            e,
                            "failed to respond message in check_in_group",
                        )
                    })?;

                return Ok(());
            }
        }
    };
}

#[macro_export]
macro_rules! check_senders {
    ($message: ident, $state: ident) => {
        let users = &$state.env.telegram_user.users;

        if let Some(sender) = $message.sender() {
            if let Some(username) = sender.username() {
                if users.len() > 0 && !users.contains(&username.to_string()) {
                    return Ok(());
                }
            }
        }
    };
}
