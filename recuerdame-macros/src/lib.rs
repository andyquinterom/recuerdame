extern crate proc_macro;

use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, ItemFn, Meta, Pat, Token, Visibility, parse_macro_input, punctuated::Punctuated};

/// Precalculate all possible values for const function at compile time.
///
/// This macro builds a look-up table at compile time to avoid
/// having to run complicated arithmentic at runtime.
///
/// This macro supports three operating modes:
///  - **basic**: The basic operating mode will limit the input range of the function to the ranges specified. If input outside the ranges is provided it will panic.
///  - **option**: The option operating mode will change the function to return an [Option]. [Some] if the input is in range, [None] if not.
///  - **keep**: The keep operating mode never panic (unless the implementation panics). It will use the look up table for the specified ranges and use the original implementation if outside of the range.
///
/// The option and keep modes will require additional bounds checks which may come at a cost.
///
/// Please benchmark the functions to decide if it's worth using a look-up table.
///
/// Examples:
/// ```rust
/// use recuerdame::precalculate;
///
/// #[precalculate(a = 0..=10, b = 0..=4)]
/// pub const fn add(a: i32, b: i32) -> i32 {
///     a + b
/// }
///
/// #[precalculate(a = 0..=10, b = 0..=4, option)]
/// pub const fn add_opt(a: i32, b: i32) -> i32 {
///     a + b
/// }
///
/// #[precalculate(a = 0..=10, b = 0..=4, keep)]
/// pub const fn add_keep(a: i32, b: i32) -> i32 {
///     a + b
/// }
///
/// #[test]
/// fn it_works() {
///     assert_eq!(add(8, 2), 10);
///     assert_eq!(add(0, 0), 0);
/// }
///
/// #[test]
/// fn it_works_opt() {
///     assert_eq!(add_opt(5, 4), Some(9));
///     assert_eq!(add_opt(25, 0), None);
/// }
///
/// #[test]
/// fn it_works_keep() {
///     assert_eq!(add_keep(5, 4), 9);
///     assert_eq!(add_keep(25, 0), 25);
/// }
///
/// #[test]
/// #[should_panic]
/// fn outside_bounds_panics() {
///     add(25, 9);
/// }
/// ```
#[proc_macro_attribute]
pub fn precalculate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let metas: Punctuated<Meta, Token![,]> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    #[derive(Debug, Hash, PartialEq, Eq)]
    enum Options {
        Option,
        KeepOriginal,
    }

    let mut options = HashSet::new();
    let mut range_map = HashMap::<String, proc_macro2::TokenStream>::new();
    for meta in metas {
        match meta {
            Meta::NameValue(mnv) => {
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
            Meta::Path(opt) => {
                match opt.to_token_stream().to_string().trim() {
                    "option" => {
                        options.insert(Options::Option);
                    }
                    "keep" => {
                        options.insert(Options::KeepOriginal);
                    }
                    opt => panic!("Unknown option: {opt}"),
                };
            }
            _ => (),
        }
    }

    if options.contains(&Options::Option) && options.contains(&Options::KeepOriginal) {
        panic!("precalculate macro may only take `option` or `keep` exclusively.")
    }

    let mut func = parse_macro_input!(item as ItemFn);
    let visibility = func.vis.clone();
    let func_ident = func.sig.ident.clone();
    let new_func_ident = format_ident!("_{func_ident}_original");
    func.vis = Visibility::Public(syn::token::Pub::default());
    func.sig.ident = new_func_ident.clone();
    let func_return_type = &func.sig.output;
    let mut return_ty = match func_return_type {
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

    let func_args = arg_info.iter().map(|(ident, _, _)| ident);

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
            let table_access = arg_info
                .iter()
                .fold(quote! { table }, |acc, (ident, _, _)| {
                    let loop_var = format_ident!("{}_idx", ident);
                    quote! { #acc[#loop_var] }
                });

            let func_args = func_args.clone();

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

        let bounds_check_expr = {
            let per_ident_check = arg_info.iter().map(|(ident, _ty, _)| {
                let min_ident = format_ident!("{}_MIN", ident.to_string().to_uppercase());
                let max_ident = format_ident!("{}_MAX", ident.to_string().to_uppercase());
                quote! { #min_ident <= #ident && #ident <= #max_ident }
            });

            quote! { #(#per_ident_check &&)* true }
        };

        let mut table_access =
            arg_info
                .iter()
                .fold(quote! { #lookup_table_ident }, |acc, (ident, _, _)| {
                    let index_var = format_ident!("{}_idx", ident);
                    quote! { #acc[#index_var] }
                });

        let opt_check = {
            options.contains(&Options::Option).then(|| {
                *return_ty.as_mut() = syn::Type::Verbatim(quote! { Option<#return_ty> });
                table_access = quote! { Some(#table_access)};
                quote! {
                    if !(#bounds_check_expr) {
                        return None;
                    }
                }
            })
        };

        let keep_check = {
            options.contains(&Options::KeepOriginal).then(|| {
                quote! {
                    if !(#bounds_check_expr) {
                        return #new_func_ident(#(#func_args),*);
                    }
                }
            })
        };

        quote! {
            pub const fn #func_ident(#(#fn_params),*) -> #return_ty {
                #opt_check
                #keep_check
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

        #[allow(unused_imports)]
        #visibility use #mod_name::#func_ident;
    };

    expanded.into()
}
