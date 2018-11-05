// Workaround for 2018 imports
extern crate proc_macro;
use self::proc_macro::{TokenStream, TokenTree};

use http::Method;
use quote::quote;
use std::cell::RefCell;
use syn::*;

pub struct RouteInfo {
    method: http::Method,
    path: String,
}

thread_local! {
    static ROUTES: RefCell<Vec<RouteInfo>> = RefCell::new(Vec::new());
}

///TODO: Better errors
pub fn route_impl(attr: TokenStream, function: TokenStream) -> TokenStream {
    let ItemFn {
        ident, block, decl, ..
    } = match syn::parse(function.clone()).expect("failed to parse tokens as a function") {
        Item::Fn(item) => item,
        _ => panic!("#[route] can only be applied to functions"),
    };

    let attr = TokenStream::from(attr);
    let tokens: Vec<TokenTree> = attr.into_iter().collect();

    let method = match &tokens[0] {
        TokenTree::Ident(ident) => {
            let ident = ident.to_string();

            match &*ident {
                "GET" => Method::GET,
                "POST" => Method::POST,
                "PUT" => Method::PUT,
                "DELETE" => Method::DELETE,
                "HEAD" => Method::HEAD,
                "OPTIONS" => Method::OPTIONS,
                "CONNECT" => Method::CONNECT,
                "PATCH" => Method::PATCH,
                "TRACE" => Method::TRACE,
                _ => panic!("{} is not a valid HTTP method", ident),
            }
        }
        _ => panic!("the first argument must be an identifier"),
    };

    let path = match &tokens[2] {
        // TODO: Verify if valid url
        TokenTree::Literal(l) => l.clone(),
        _ => panic!("the second argument must be a string literal"),
    };

    let route = RouteInfo {
        method,
        path: path.to_string(),
    };

    ROUTES.with(|f| {
        f.borrow_mut().push(route);
    });

    let inputs = decl.inputs;
    let output = decl.output;

    // TODO: This nessecary?
    // syn doesn't know how to parse async functions yet, so for now, we don't write async
    // and this re-constructs the function with the async in front, and without the attribute
    let tokens = quote! {
        async fn #ident (#inputs) #output #block
    };

    tokens.into()
}
