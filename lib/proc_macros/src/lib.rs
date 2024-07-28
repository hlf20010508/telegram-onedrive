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
pub fn add_trace(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = args.to_string();

    if !args.is_empty() && args != "context" {
        return quote! { compile_error!("only accept `context` as argument"); }.into();
    }

    let should_add_context = args == "context";

    let input = parse_macro_input!(item as ItemFn);

    let fn_visibility = &input.vis;
    let fn_is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_generics = &input.sig.generics;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_where_clause = &input.sig.generics.where_clause;
    let fn_block = &input.block;

    let fn_name_str = &fn_name.to_string();

    let fn_sign_part = quote! {
        #fn_visibility fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause
    };
    let async_fn_sign_part = quote! {
        #fn_visibility async fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause
    };

    let fn_return_part = quote! {(|| { #fn_block })()};
    let async_fn_return_part = quote! {(async #fn_block).await};

    let trace_part = quote! {
        let func_path = module_path!().to_string() + "::" + #fn_name_str;
        tracing::trace!("{}", func_path);
    };
    let context_part = quote! {
        use crate::error::ResultExt;
    };
    let context_part_return = quote! {context(func_path)};

    let expanded = if fn_is_async {
        if should_add_context {
            quote! {
                #async_fn_sign_part {
                    #context_part
                    #trace_part
                    #async_fn_return_part.#context_part_return
                }
            }
        } else {
            quote! {
                #async_fn_sign_part {
                    #trace_part
                    #async_fn_return_part
                }
            }
        }
    } else if should_add_context {
        quote! {
            #fn_sign_part {
                #context_part
                #trace_part
                #fn_return_part.#context_part_return
            }
        }
    } else {
        quote! {
            #fn_sign_part {
                #trace_part
                #fn_return_part
            }
        }
    };

    TokenStream::from(expanded)
}
