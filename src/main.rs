use axum::{
    Router, 
    response::Html, 
    routing::get, extract::Path
};
use pulldown_cmark::{
    Options, 
    Parser, 
    html
};
use reqwest::StatusCode;

#[macro_use]
extern crate lazy_static;

mod github;
mod utils;

/// this flag is to set up debugging instances to allow self-signed certificates.
#[cfg(not(debug_assertions))]
pub (crate) const ACCEPT_INVALID_CERTS: bool = false;
#[cfg(debug_assertions)]
pub (crate) const ACCEPT_INVALID_CERTS: bool = true;

#[cfg(not(debug_assertions))]
const INDEX_URL: &'static str = "adamfortune.com";
#[cfg(debug_assertions)]
const INDEX_URL: &'static str = "http://localhost:3000";

const ERROR_RESPONSE: &'static str = "Failed to reach github to fetch resources.";

const ALL_PAGES_CSS: &'static str = r#"
html, body {
    margin: 0;
    padding: 0;
    border: 0;
    outline: 0;
    font-size: 100%;
    vertical-align: baseline;
    background: #191919;
    color: #FFFFFF;
}
body {
    padding: 0px 18px;
}
#navbar_list > li {
    display: inline-block;
    padding: 16px 0px;
}
#navbar_list > li > a {
    color: #FFFFFF;
    padding: 10px;
    text-decoration: none;
}
a {
    color: #daa214;
}
code {
    max-width: 100%;
    overflow: scroll;
    display: inline-block;
}
h1 {
    text-align: center;
}
"#;

const CONTENT_LIST_CSS: &'static str = r#"
li {
    list-style: none;
}
"#;

const MARKDOWN_CSS: &'static str = r#"
"#;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/projects", get(projects))
        .route("/projects/:project", get(project))
        .route("/blog", get(blog));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<String> {
    match github::get_data().await {
        Err(_) => {
            Html(ERROR_RESPONSE.to_string())
        }
        Ok(data) => {
            let mut html = create_html_page(false);
            html.push_str("<body onLoad='onLoad()'>"); {
                

                html.push_str(&create_nav_bar(None));
                html.push_str("<div style='margin-left:8px;'>"); {
                    html.push_str(&parse_md_to_html(&data.home.content));
                }
                html.push_str("</div>");
            }
            html.push_str("</body>");
            Html(html)
        }
    }
}

async fn projects() -> Html<String> {
    match github::get_data().await {
        Err(_) => Html(ERROR_RESPONSE.to_string()),
        Ok(data) => {
            let mut html = create_html_page(true);
            html.push_str("<body onLoad='onLoad()'>"); {
                html.push_str(&create_nav_bar(None));

                html.push_str("<ul>");
                for repo in data.repos {
                    html.push_str("<li>"); {
                        html.push_str("<h2>"); {
                            html.push_str(&format!("<a href='{}/projects/{}'>", INDEX_URL, get_url_safe_name(&repo.name))); {
                                html.push_str(&repo.name);
                            }
                            html.push_str("</a>");
                        }
                        html.push_str("</h2>");

                        html.push_str("<p>"); {
                            html.push_str(&repo.description);
                        }
                        html.push_str("</p>");
                    }
                    html.push_str("</li>");
                }
                html.push_str("</ul>");
            }
            html.push_str("</body>");
            Html(html)
        }
    }
}

async fn project(Path(project): Path<String>) -> Result<Html<String>, StatusCode> {
    match github::get_data().await {
        Err(_) => Ok(Html(ERROR_RESPONSE.to_string())),
        Ok(data) => {
            match data.repos.into_iter().find(|repo| get_url_safe_name(&repo.name) == get_url_safe_name(&project)) {
                None => Err(StatusCode::NOT_FOUND),
                Some(repo) => {
                    let mut html = create_html_page(false);
                    html.push_str("<body onLoad='onLoad()'>");
                    html.push_str(&create_nav_bar(Some(&repo.html_url)));
                    match repo.readme {
                        None => {
                            html.push_str("</body>");
                            Ok(Html(html))
                        }
                        Some(readme) => {
                            html.push_str("<div>"); {
                                html.push_str(&parse_md_to_html(&readme));
                            }
                            html.push_str("</div>");
                            html.push_str("</body>");
                            Ok(Html(html))
                        }
                    }
                }
            }
        }
    }
}

async fn blog() -> Html<String> {
    let mut html = create_html_page(true);
    html.push_str("<body onLoad='onLoad()'>"); {
        html.push_str(&create_nav_bar(None));
    }
    html.push_str("</body>");
    Html(html)
}

/// Creates an HTML page, adding the <head> tag that is needed. 
/// Callers should add the <body> tag and all inner content
fn create_html_page(is_content_list: bool) -> String {
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str("<head>"); {
        html.push_str("<script>"); {
            html.push_str(
                r#"
                function onLoad() {
                    mermaidNodes = document.querySelectorAll("pre > code.language-mermaid");
                    // the formatter I'm using doesn't quite do what mermaid is expecting, so let's fix that by moving the class "mermaid" to the "pre" element.
                    for (let i = 0; i < mermaidNodes.length; i++) {                            
                        mermaidNodes[i].parentNode.classList.add("mermaid");
                        mermaidNodes[i].parentNode.innerHTML = mermaidNodes[i].innerHTML;
                    }
                }
                "#
            )
        }
        html.push_str("</script>");
        html.push_str("<script type='module'>"); {
            html.push_str("import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';");
        }
        html.push_str("</script>");

        html.push_str("<style>"); {
            html.push_str(ALL_PAGES_CSS);
            if is_content_list {
                html.push_str(CONTENT_LIST_CSS);
            } else {
                html.push_str(MARKDOWN_CSS);
            }
        }
        html.push_str("</style>");
    }
    html.push_str("</head>");
    html
}

fn create_nav_bar(source_code_url: Option<&str>) -> String {
    let mut html = String::new();
    html.push_str("<nav id='navbar'>"); {
        html.push_str("<ul id='navbar_list' style='list-style: none; display: flex; flex-direction: row; justify-content: left; margin: 0px; padding: 0px;'>"); {
            html.push_str("<li>"); {
                html.push_str("<a href='");
                html.push_str(INDEX_URL);
                html.push_str("/'>Home</a>");
            }
            html.push_str("<li>"); {
                html.push_str("<a href='");
                html.push_str(INDEX_URL);
                html.push_str("/projects'>Projects</a>");
            }
            html.push_str("<li>"); {
                html.push_str("<a href='");
                html.push_str(INDEX_URL);
                html.push_str("/blog'>Blog</a>");
            }
            if let Some(source_code_url) = source_code_url {
                html.push_str("<li>"); {
                    html.push_str("<a href='");
                    html.push_str(source_code_url);
                    html.push_str("'>Source Code</a>");
                }
            }
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }
    html.push_str("</nav>");
    html
}

fn get_url_safe_name(name: &str) -> String {
    return name.chars()
        .filter(|char| {
            match char {
                '0'..='9'
                | 'a'..='z'
                | 'A'..='Z'
                | '_' => true,
                _ => false,
            }
        }).collect()
}

fn parse_md_to_html(md: &str) -> String {
    let parser = Parser::new_ext(&md, Options::empty());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}