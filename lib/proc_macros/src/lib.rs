/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn add_trace(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_attrs = &input.attrs;
    let fn_visibility = &input.vis;
    let fn_is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_generics = &input.sig.generics;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_where_clause = &input.sig.generics.where_clause;
    let fn_block = &input.block;

    let fn_name_str = &fn_name.to_string();

    let expanded = if fn_is_async {
        quote! {
            #(#fn_attrs)*
            #fn_visibility async fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause {
                let func_path = module_path!().to_string() + "::" + #fn_name_str;
                tracing::trace!("{}", func_path);
                #fn_block
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #fn_visibility fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause {
                let func_path = module_path!().to_string() + "::" + #fn_name_str;
                tracing::trace!("{}", func_path);
                #fn_block
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn add_context(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_attrs = &input.attrs;
    let fn_visibility = &input.vis;
    let fn_is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_generics = &input.sig.generics;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_where_clause = &input.sig.generics.where_clause;
    let fn_block = &input.block;

    let fn_name_str = &fn_name.to_string();

    let expanded = if fn_is_async {
        quote! {
            #(#fn_attrs)*
            #fn_visibility async fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause {
                use crate::error::ResultExt;
                let func_path = module_path!().to_string() + "::" + #fn_name_str;
                async #fn_block.await.context(func_path)
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #fn_visibility fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause {
                use crate::error::ResultExt;
                let func_path = module_path!().to_string() + "::" + #fn_name_str;
                (|| #fn_block )().context(func_path)
            }
        }
    };

    TokenStream::from(expanded)
}

macro_rules! gen_checker {
    ($marcro_name:ident, $code:block) => {
        #[proc_macro_attribute]
        pub fn $marcro_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
            let input = parse_macro_input!(item as ItemFn);

            let fn_attrs = &input.attrs;
            let fn_name = &input.sig.ident;
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

            let fn_output = &input.sig.output;
            let fn_block = &input.block;

            let expanded = quote! {
                #(#fn_attrs)*
                pub async fn #fn_name(#fn_inputs) #fn_output {
                    $code

                    #fn_block
                }
            };

            TokenStream::from(expanded)
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
                .details(CHECK_IN_GROUP_FAILED)?;

            return Ok(());
        }
    }
});

gen_checker!(check_senders, {
    let users = &state.env.telegram_user.users;

    if let Some(sender) = message.sender() {
        if let Some(username) = sender.username() {
            if users.len() > 0 && !users.contains(&username.to_string()) {
                return Ok(());
            }
        }
    }
});

gen_checker!(check_tg_login, {
    let is_authorized = state.telegram_user.is_authorized().await?;

    if !is_authorized {
        let response = "You haven't logged in to Telegram.";
        message.respond(response).await.details(response)?;

        let env = &state.env;
        let _server_abort_handle = crate::auth_server::spawn(env).await?;
        crate::handlers::auth::login_to_telegram(message.clone(), state.clone()).await?;
    }
});

gen_checker!(check_od_login, {
    let is_authorized = state.onedrive.is_authorized().await;

    if !is_authorized {
        let response = "You haven't authorize OneDrive.";
        message.respond(response).await.details(response)?;

        let env = &state.env;
        let _server_abort_handle = crate::auth_server::spawn(env).await?;
        crate::handlers::auth::authorize_onedrive(message.clone(), state.clone(), false).await?;
    }
});
