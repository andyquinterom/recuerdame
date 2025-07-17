use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Expr, ItemFn, Lit, Meta, MetaNameValue, PatLit, Token, parse_macro_input};

#[proc_macro_attribute]
pub fn precalculate(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute as a TokenStream2, then as a comma-separated list of Meta
    let metas: syn::punctuated::Punctuated<Meta, Token![,]> =
        parse_macro_input!(attr with syn::punctuated::Punctuated::parse_terminated);

    let mut range_map = HashMap::<String, proc_macro2::TokenStream>::new();

    for meta in metas {
        if let Meta::NameValue(mnv) = meta {
            let ident = mnv.path.get_ident().map(|i| i.to_string()).unwrap();
            let lit = mnv.value;
            let expr = lit.to_token_stream();
            if range_map.insert(ident.clone(), expr).is_some() {
                panic!("Duplicated key: {ident}")
            };
        }
    }

    // parse the function which we are giving this attribute to and save its arguments into the set
    let item_for_parsing = item.clone();
    let func = parse_macro_input!(item_for_parsing as ItemFn);
    let mut arg_set = HashSet::<String>::new();

    for arg in &func.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                arg_set.insert(pat_ident.ident.to_string());
            }
        }
    }

    for argument in arg_set {
        if !range_map.contains_key(&argument) {
            panic!("Key: {argument} does not have a specified range.")
        }
    }

    let item: proc_macro2::TokenStream = item.into();

    let expanded = quote! {
        const _: () = {
        };

        #item
    };

    expanded.into()
}
