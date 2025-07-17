extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, ItemFn, Meta, Pat, Token, Visibility, parse_macro_input, punctuated::Punctuated};

/// Precalculate all possible values for const function at compile time.
///
/// This macro builds a look-up table at compile time to avoid
/// having to run complicated arithmentic at runtime.
///
/// Please benchmark your functions to decide if it's worth using a look-up table.
///
/// Examples:
/// ```rust
/// use recuerdame::precalculate;
///
/// #[precalculate(a = 0..=10, b = 0..=4)]
/// pub const fn add(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
#[proc_macro_attribute]
pub fn precalculate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let metas: Punctuated<Meta, Token![,]> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    let mut range_map = HashMap::<String, proc_macro2::TokenStream>::new();
    for meta in metas {
        if let Meta::NameValue(mnv) = meta {
            let ident = mnv
                .path
                .get_ident()
                .expect("Attribute key must be an identifier")
                .to_string();
            let value_expr = mnv.value.into_token_stream();
            if range_map.insert(ident.clone(), value_expr).is_some() {
                panic!("Duplicated key: {ident}");
            }
        }
    }

    let mut func = parse_macro_input!(item as ItemFn);
    let visibility = func.vis.clone();
    let func_ident = func.sig.ident.clone();
    let new_func_ident = format_ident!("_{func_ident}_original");
    func.vis = Visibility::Public(syn::token::Pub::default());
    func.sig.ident = new_func_ident.clone();
    let func_return_type = &func.sig.output;
    let return_ty = match func_return_type {
        syn::ReturnType::Default => panic!("Function must have a return type."),
        syn::ReturnType::Type(_, ty) => ty.clone(),
    };

    let mut arg_info = Vec::new();
    for arg in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            let arg_name = pat_ident.ident.to_string();
            let arg_type = &pat_type.ty;
            if let Some(range_expr) = range_map.get(&arg_name) {
                arg_info.push((
                    pat_ident.ident.clone(),
                    arg_type.clone(),
                    range_expr.clone(),
                ));
            } else {
                panic!("Argument '{arg_name}' does not have a specified range.");
            }
        }
    }

    let const_defs = arg_info.iter().map(|(ident, ty, range_expr)| {
        let upper_ident = ident.to_string().to_uppercase();
        let range_ident = format_ident!("{}_RANGE", upper_ident);
        let min_ident = format_ident!("{}_MIN", upper_ident);
        let max_ident = format_ident!("{}_MAX", upper_ident);
        let size_ident = format_ident!("{}_SIZE", upper_ident);

        quote! {
            const #range_ident: std::ops::RangeInclusive<#ty> = #range_expr;
            const #min_ident: #ty = *#range_ident.start();
            const #max_ident: #ty = *#range_ident.end();
            const #size_ident: usize = (#max_ident as isize - #min_ident as isize + 1) as usize;
        }
    });

    let table_type = arg_info
        .iter()
        .rev()
        .fold(quote! { #return_ty }, |inner, (ident, _, _)| {
            let size_ident = format_ident!("{}_SIZE", ident.to_string().to_uppercase());
            quote! { [#inner; #size_ident] }
        });

    let generate_table_fn = {
        let table_init_value = quote! { recuerdame::PrecalcConst::DEFAULT };
        let table_init_expr =
            arg_info
                .iter()
                .rev()
                .fold(table_init_value, |inner, (ident, _, _)| {
                    let size_ident = format_ident!("{}_SIZE", ident.to_string().to_uppercase());
                    quote! { [#inner; #size_ident] }
                });

        let mut nested_loops = {
            let value_calcs = arg_info.iter().map(|(ident, ty, _)| {
                let min_ident = format_ident!("{}_MIN", ident.to_string().to_uppercase());
                let loop_var = format_ident!("{}_idx", ident);
                quote! { let #ident = #min_ident + #loop_var as #ty; }
            });
            let func_args = arg_info.iter().map(|(ident, _, _)| ident);
            let table_access = arg_info
                .iter()
                .fold(quote! { table }, |acc, (ident, _, _)| {
                    let loop_var = format_ident!("{}_idx", ident);
                    quote! { #acc[#loop_var] }
                });

            quote! {
                #(#value_calcs)*
                #table_access = #new_func_ident(#(#func_args),*);
            }
        };

        for (ident, _, _) in arg_info.iter().rev() {
            let loop_var = format_ident!("{}_idx", ident);
            let size_ident = format_ident!("{}_SIZE", ident.to_string().to_uppercase());
            nested_loops = quote! {
                let mut #loop_var: usize = 0;
                while #loop_var < #size_ident {
                    #nested_loops
                    #loop_var += 1;
                }
            };
        }

        quote! {
            const fn generate_table() -> #table_type {
                let mut table = #table_init_expr;
                #nested_loops
                table
            }
        }
    };

    let mod_name = format_ident!("_mod_precalc_{}", func_ident);

    let precalc_fn = {
        let lookup_table_ident =
            format_ident!("LOOKUP_TABLE_{}", func_ident.to_string().to_uppercase());

        let fn_params = arg_info.iter().map(|(ident, ty, _)| quote! { #ident: #ty });
        let index_calcs = arg_info.iter().map(|(ident, _ty, _)| {
            let min_ident = format_ident!("{}_MIN", ident.to_string().to_uppercase());
            let index_var = format_ident!("{}_idx", ident);
            quote! { let #index_var = (#ident - #min_ident) as usize; }
        });
        let table_access =
            arg_info
                .iter()
                .fold(quote! { #lookup_table_ident }, |acc, (ident, _, _)| {
                    let index_var = format_ident!("{}_idx", ident);
                    quote! { #acc[#index_var] }
                });

        quote! {
            pub const fn #func_ident(#(#fn_params),*) -> #return_ty {
                #(#index_calcs)*
                #table_access
            }
        }
    };

    let lookup_table_ident =
        format_ident!("LOOKUP_TABLE_{}", func_ident.to_string().to_uppercase());
    let expanded = quote! {

        mod #mod_name {

            use super::*;

            #func

            #(#const_defs)*

            #generate_table_fn

            pub const #lookup_table_ident: &'static #table_type = &generate_table();

            #precalc_fn
        }

        #visibility use #mod_name::#func_ident;
    };

    expanded.into()
}
