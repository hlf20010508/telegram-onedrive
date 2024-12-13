/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

macro_rules! gen_checker {
    ($marcro_name:ident, $code:block) => {
        #[proc_macro_attribute]
        pub fn $marcro_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
            let input = parse_macro_input!(item as ItemFn);

            let fn_inputs = &input.sig.inputs;

            if fn_inputs.len() != 2 {
                return quote! {
                    compile_error!("only works with 2 arguments");
                }
                .into();
            }

            let param_names = input
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        if let syn::Pat::Ident(ident) = &*pat_type.pat {
                            Some(ident.ident.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();

            let expected_param_names = ["message", "state"];

            for expected in expected_param_names {
                if !param_names.contains(&expected.to_string()) {
                    return quote! {
                        compile_error!(concat!("expect parameter name: ", #expected));
                    }
                    .into();
                }
            }

            let fn_attrs = &input.attrs;
            let fn_visibility = &input.vis;
            let fn_sig = &input.sig;
            let fn_block = &input.block;

            let tokens = quote_spanned! {input.span() =>
                #(#fn_attrs)*
                #fn_visibility #fn_sig {
                    $code

                    #fn_block
                }
            };

            tokens.into()
        }
    };
}

gen_checker!(check_in_group, {
    match message.chat() {
        grammers_client::types::Chat::Group(_) => {}
        _ => {
            const CHECK_IN_GROUP_FAILED: &str = r"
This bot must be used in a Group!

Add this bot to a Group as Admin, and give it ability to Delete Messages.
";

            message
                .respond(CHECK_IN_GROUP_FAILED)
                .await
                .context(CHECK_IN_GROUP_FAILED)?;

            return Ok(());
        }
    }
});

gen_checker!(check_senders, {
    let users = &crate::env::ENV.get().unwrap().telegram_user.users;

    if let Some(sender) = message.sender() {
        if let Some(username) = sender.username() {
            if !users.is_empty() && !users.contains(&username.to_string()) {
                return Ok(());
            }
        }
    }
});

gen_checker!(check_tg_login, {
    let is_authorized = state.telegram_user.is_authorized().await?;

    if !is_authorized {
        let response = "You haven't logged in to Telegram.";
        message.respond(response).await.context(response)?;

        let (rx, _, _server_abort_handle) = crate::auth_server::spawn().await?;
        crate::handlers::auth::login_to_telegram(message.clone(), state.clone(), rx).await?;
    }
});

gen_checker!(check_od_login, {
    let is_authorized = state.onedrive.is_authorized().await;

    if !is_authorized {
        let response = "You haven't authorize OneDrive.";
        message.respond(response).await.context(response)?;

        let (_, rx, _server_abort_handle) = crate::auth_server::spawn().await?;
        crate::handlers::auth::authorize_onedrive(message.clone(), state.clone(), false, rx)
            .await?;
    }
});
