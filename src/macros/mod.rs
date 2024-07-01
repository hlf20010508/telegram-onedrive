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
                        crate::error::Error::details(
                            e,
                            "failed to respond message",
                            crate::macros::docs::CHECK_IN_GROUP_FAILED,
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

#[macro_export]
macro_rules! check_tg_login {
    ($message: ident, $state: ident) => {
        let is_authorized = $state
            .telegram_user
            .client
            .is_authorized()
            .await
            .map_err(|e| {
                Error::context(
                    e,
                    "failed to check telegram user client authorization state",
                )
            })?;

        if !is_authorized {
            let response = "You haven't logined to Telegram.";
            $message
                .respond(response)
                .await
                .map_err(|e| Error::respond_error(e, response))?;

            let env = &$state.env;
            let _server_abort_handle = crate::auth_server::spawn(env).await?;
            crate::handlers::auth::login_to_telegram($message.clone(), $state.clone()).await?;
        }
    };
}

#[macro_export]
macro_rules! check_od_login {
    ($message: ident, $state: ident) => {
        let is_authorized = $state.onedrive.is_authorized().await;

        if !is_authorized {
            let response = "You haven't authorize OneDrive.";
            $message
                .respond(response)
                .await
                .map_err(|e| Error::respond_error(e, response))?;

            let env = &$state.env;
            let _server_abort_handle = crate::auth_server::spawn(env).await?;
            crate::handlers::auth::authorize_onedrive($message.clone(), $state.clone(), false)
                .await?;
        }
    };
}
