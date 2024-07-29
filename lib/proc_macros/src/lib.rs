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
