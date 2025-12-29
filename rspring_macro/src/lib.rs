use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr, ItemStruct, Type};

// #[service] and #[controller] do the SAME thing
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component_macro(item)
}

#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component_macro(item)
}

fn component_macro(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let name_str = name.to_string();
    
    // Extract dependencies from fields
    let mut deps = Vec::new();
    if let syn::Fields::Named(fields) = &input.fields {
        for field in &fields.named {
            if let Type::Path(type_path) = &field.ty {
                // Check if it's Arc<SomeType>
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Arc" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                deps.push(quote! { std::any::TypeId::of::<#inner_ty>() });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Make deps a const array
    let deps_array = if deps.is_empty() {
        quote! { &[] }
    } else {
        quote! { &[#(#deps),*] }
    };
    
    // Generate field initializations
    let field_inits = if let syn::Fields::Named(fields) = &input.fields {
        fields.named.iter().map(|field| {
            let field_name = &field.ident;
            if let Type::Path(type_path) = &field.ty {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Arc" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return quote! { #field_name: container.get::<#inner_ty>() };
                            }
                        }
                    }
                }
            }
            quote! { #field_name: Default::default() }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };
    
    quote! {
        #input
        
        rspring::inventory::submit! {
            rspring::Component {
                name: #name_str,
                type_id: std::any::TypeId::of::<#name>(),
                dependencies: #deps_array,
                build: |container| {
                    std::sync::Arc::new(#name {
                        #(#field_inits),*
                    })
                }
            }
        }
    }
    .into()
}

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