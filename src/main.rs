use axum::{
    Router, 
    response::Html, 
    routing::get
};
use pulldown_cmark::{
    Options, 
    Parser, 
    html
};

#[macro_use]
extern crate lazy_static;

mod github;
mod utils;

/// this flag is to set up debugging instances to allow self-signed certificates.
#[cfg(not(debug_assertions))]
pub (crate) const ACCEPT_INVALID_CERTS: bool = false;
#[cfg(debug_assertions)]
pub (crate) const ACCEPT_INVALID_CERTS: bool = true;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<String> {
    match github::get_data().await {
        Err(_) => {
            Html("Failed to reach github to fetch resources.".to_string())
        }
        Ok(data) => {
            let html = data
                .iter()
                .map(|repo| {
                    let parser;
                    match &repo.readme {
                        None => return String::with_capacity(0),
                        Some(readme) => {
                            parser = Parser::new_ext(&readme, Options::empty());
                        }
                    }
                    
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    html_output
                }).collect::<Vec<String>>()
                .join("");
            Html(html)
        }
    }
}