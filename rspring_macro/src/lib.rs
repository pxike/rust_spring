use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr, ItemStruct, Type, ItemImpl, ImplItem, FnArg};

// #[service] and #[controller] do the SAME thing
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try parsing as struct first
    if let Ok(_) = syn::parse::<ItemStruct>(item.clone()) {
        return component_macro(item);
    }
    
    // Try parsing as impl
    if let Ok(input) = syn::parse::<ItemImpl>(item) {
        return service_impl_macro(input);
    }

    panic!("#[service] can only be used on structs or impl blocks");
}

#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try parsing as struct first
    if let Ok(_) = syn::parse::<ItemStruct>(item.clone()) {
        return component_macro(item);
    }
    
    // Try parsing as impl
    if let Ok(input) = syn::parse::<ItemImpl>(item) {
        return controller_impl_macro(input);
    }

    panic!("#[controller] can only be used on structs or impl blocks");
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

fn service_impl_macro(input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    let mut generated_items = Vec::new();
    
    // 1. Check for a `new` method to generate Component registration (Constructor Injection)
    let mut component_registration = None;
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            if method.sig.ident == "new" {
                let name_str = quote!(#self_ty).to_string();
                
                // Extract dependencies from `new` arguments
                let mut deps = Vec::new();
                let mut inject_calls = Vec::new();
                
                for arg in &method.sig.inputs {
                    if let FnArg::Typed(pat_type) = arg {
                        if let Type::Path(type_path) = &*pat_type.ty {
                            // Check if it's Arc<SomeType>
                            if let Some(segment) = type_path.path.segments.last() {
                                if segment.ident == "Arc" {
                                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                            deps.push(quote! { std::any::TypeId::of::<#inner_ty>() });
                                            inject_calls.push(quote! { container.get::<#inner_ty>() });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                let deps_array = if deps.is_empty() {
                    quote! { &[] }
                } else {
                    quote! { &[#(#deps),*] }
                };
                
                component_registration = Some(quote! {
                    rspring::inventory::submit! {
                        rspring::Component {
                            name: #name_str,
                            type_id: std::any::TypeId::of::<#self_ty>(),
                            dependencies: #deps_array,
                            build: |container| {
                                std::sync::Arc::new(#self_ty::new(
                                    #(#inject_calls),*
                                ))
                            }
                        }
                    }
                });
            }
        }
    }
    
    if let Some(reg) = component_registration {
        generated_items.push(reg);
    }
    
    quote! {
        #input
        #(#generated_items)*
    }.into()
}

fn controller_impl_macro(mut input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    let mut generated_items = Vec::new();
    
    // 1. Check for a `new` method to generate Component registration (Constructor Injection)
    let mut component_registration = None;
    
    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            if method.sig.ident == "new" {
                let name_str = quote!(#self_ty).to_string();
                
                // Extract dependencies from `new` arguments
                let mut deps = Vec::new();
                let mut inject_calls = Vec::new();
                
                for arg in &method.sig.inputs {
                    if let FnArg::Typed(pat_type) = arg {
                        if let Type::Path(type_path) = &*pat_type.ty {
                            // Check if it's Arc<SomeType>
                            if let Some(segment) = type_path.path.segments.last() {
                                if segment.ident == "Arc" {
                                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                            deps.push(quote! { std::any::TypeId::of::<#inner_ty>() });
                                            inject_calls.push(quote! { container.get::<#inner_ty>() });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                let deps_array = if deps.is_empty() {
                    quote! { &[] }
                } else {
                    quote! { &[#(#deps),*] }
                };
                
                component_registration = Some(quote! {
                    rspring::inventory::submit! {
                        rspring::Component {
                            name: #name_str,
                            type_id: std::any::TypeId::of::<#self_ty>(),
                            dependencies: #deps_array,
                            build: |container| {
                                std::sync::Arc::new(#self_ty::new(
                                    #(#inject_calls),*
                                ))
                            }
                        }
                    }
                });
            }
        }
    }
    
    if let Some(reg) = component_registration {
        generated_items.push(reg);
    }

    // 2. Process Routes
    for item in &mut input.items {
        if let ImplItem::Fn(method) = item {
            let mut route_attr = None;
            let mut attr_idx_to_remove = None;
            
            for (i, attr) in method.attrs.iter().enumerate() {
                if attr.path().is_ident("get") || attr.path().is_ident("post") || 
                   attr.path().is_ident("put") || attr.path().is_ident("delete") {
                    route_attr = Some(attr.clone());
                    attr_idx_to_remove = Some(i);
                    break;
                }
            }
            
            if let Some(attr) = route_attr {
                if let Some(idx) = attr_idx_to_remove {
                    method.attrs.remove(idx);
                }
                
                let method_name = &method.sig.ident;
                let path_lit: LitStr = attr.parse_args().expect("Invalid route path");
                
                let (http_method, axum_method) = if attr.path().is_ident("get") {
                    (quote! { rspring::Method::GET }, quote! { rspring::axum::routing::get })
                } else if attr.path().is_ident("post") {
                    (quote! { rspring::Method::POST }, quote! { rspring::axum::routing::post })
                } else if attr.path().is_ident("put") {
                    (quote! { rspring::Method::PUT }, quote! { rspring::axum::routing::put })
                } else {
                    (quote! { rspring::Method::DELETE }, quote! { rspring::axum::routing::delete })
                };

                let wrapper_name = quote::format_ident!("{}_handler", method_name);
                let mut wrapper_args = Vec::new();
                let mut call_args = Vec::new();
                
                for (i, arg) in method.sig.inputs.iter().enumerate() {
                    match arg {
                        FnArg::Receiver(_) => {}, // Skip self
                        FnArg::Typed(pat_type) => {
                            let ty = &pat_type.ty;
                            let arg_name = quote::format_ident!("arg{}", i);
                            wrapper_args.push(quote! { #arg_name: #ty });
                            call_args.push(quote! { #arg_name });
                        }
                    }
                }
                
                let output = &method.sig.output;
                
                generated_items.push(quote! {
                    async fn #wrapper_name(
                        rspring::axum::Extension(container): rspring::axum::Extension<std::sync::Arc<rspring::ServiceContainer>>,
                        #(#wrapper_args),*
                    ) #output {
                        let controller = container.get::<#self_ty>();
                        controller.#method_name(#(#call_args),*).await
                    }
                    
                    rspring::inventory::submit! {
                        rspring::Route {
                            path: #path_lit,
                            method: #http_method,
                            setup: |router| {
                                router.route(#path_lit, #axum_method(#wrapper_name))
                            }
                        }
                    }
                });
            }
        }
    }
    
    quote! {
        #input
        #(#generated_items)*
    }.into()
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