use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    quote! {
        #input
        inventory::submit! {
            rspring::Route {
                path: #path,
                method: rspring::Method::GET,
                setup: |router| {
                    router.route(#path, axum::routing::get(#name))
                }
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    quote! {
        #input
        inventory::submit! {
            rspring::Route {
                path: #path,
                method: rspring::Method::DELETE,
                setup: |router| {
                    router.route(#path, axum::routing::delete(#name))
                }
            }
        }
    }
    .into()
}


#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    quote! {
        #input
        inventory::submit! {
            rspring::Route {
                path: #path,
                method: rspring::Method::POST,
                setup: |router| {
                    router.route(#path, axum::routing::post(#name))
                }
            }
        }
    }
    .into()
}
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    quote! {
        #input
        inventory::submit! {
            rspring::Route {
                path: #path,
                method: rspring::Method::PUT,
                setup: |router| {
                    router.route(#path, axum::routing::put(#name))
                }
            }
        }
    }
    .into()
}


#[proc_macro]
pub fn scan_controllers(_input: TokenStream) -> TokenStream {
    quote! {
        mod controllers;
    }.into()
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let body = &input.block;

    quote! {
        #[tokio::main]
        async fn #name() {
            { #body }

            let app = rspring::App::new();
            app.run().await;
        }
    }.into()
}