use axum::{
    Router, 
    response::Html, 
    routing::{get, post}, extract::{Path, State}, Form
};
use github::{Repo, BlogPost};
use pulldown_cmark::{
    Options, 
    Parser, 
    html
};
use reqwest::{StatusCode, Method};
use serde_derive::Deserialize;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

mod github;
mod utils;

/// this flag is to set up debugging instances to allow self-signed certificates.
#[cfg(not(debug_assertions))]
pub (crate) const ACCEPT_INVALID_CERTS: bool = false;
#[cfg(debug_assertions)]
pub (crate) const ACCEPT_INVALID_CERTS: bool = true;

#[cfg(not(debug_assertions))]
const INDEX_URL: &'static str = "https://adamfortunecom.shuttleapp.rs";
#[cfg(debug_assertions)]
const INDEX_URL: &'static str = "http://127.0.0.1:8000";

const ERROR_RESPONSE: &'static str = "Failed to reach database.";

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
li:not(#navbar li) {
    border: 1px solid #FFFFFF;
    padding: 20px;
    border-radius: 20px;
}
"#;

const MARKDOWN_CSS: &'static str = r#"

"#;

#[shuttle_runtime::main]
pub async fn shuttle_main (
    #[shuttle_shared_db::Postgres(local_uri = "postgresql://localhost/adamfortunecom?user=adam&password={secrets.PASSWORD}")] pool: PgPool,
    #[shuttle_secrets::Secrets] _secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations failed :(");

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);
    

    let state = AppState::new(pool);

    let app = Router::new()
        .route("/", get(index))
        .route("/projects", get(projects))
        .route("/projects/:project", get(project))
        .route("/blog", get(blog))
        .route("/blog/:blog", get(blog_post))
        .route("/formatjson", post(format_json))
        .with_state(state.clone())
        .layer(cors);

    Ok(app.into())
}

async fn index(State(state): State<AppState>) -> Html<String> {
    match github::get_home(&state).await {
        None => {
            Html(ERROR_RESPONSE.to_string())
        }
        Some(data) => {
            let mut html = create_html_page(false);
            html.push_str("<body onLoad='onLoad()'>"); {
                

                html.push_str(&create_nav_bar(None));
                html.push_str("<div style='margin-left:8px;'>"); {
                    html.push_str(&parse_md_to_html(&data.content));
                }
                html.push_str("</div>");
            }
            html.push_str("</body>");
            Html(html)
        }
    }
}

async fn projects(State(state): State<AppState>) -> Html<String> {
    match github::get_repos(&state).await {
        None => Html(ERROR_RESPONSE.to_string()),
        Some(data) => {
            let mut html = create_html_page(true);
            html.push_str("<body onLoad='onLoad()'>"); {
                html.push_str(&create_nav_bar(None));

                html.push_str("<ul style='display: grid;column-count: 2;column-gap: 20px;row-gap: 20px; margin-right: 30px'>");
                
                for (i, repo) in data.iter().enumerate() {
                    html.push_str(&generate_repo_card(i, repo));
                }

                html.push_str("</ul>");
            }
            html.push_str("</body>");
            Html(html)
        }
    }
}

async fn project(State(state): State<AppState>, Path(project): Path<String>) -> Result<Html<String>, StatusCode> {
    match github::get_repo(&state, &project).await {
        None => Err(StatusCode::NOT_FOUND),
        Some(mut repo) => {
            match repo.name.as_str() {
                "json-formatter" => {
                    if let Some(readme) = &mut repo.readme {
                        *readme = readme.replace(
                            "!Json Formatter Input Box Goes Here!", 
                            r#"<form action="/formatjson" method="post">
                                <label for="json">JSON:</label><br/>
                                <textarea id="json" name="json" style="width:100%;min-height:200px;"></textarea><br/>
                                <input type="submit" value="Submit">
                            </form> "#
                        );
                    }
                }
                _ => {} // do nothing
            }

            let mut html = create_html_page(false);
            html.push_str("<body onLoad='onLoad()'>");
            let mut additional_nav_bar_elements = 
                vec![
                    NavBarElement { 
                        display_text: "Source Code".to_string(), 
                        href: repo.html_url 
                    }
                ];

            match repo.name.as_str() {
                "tree-iterators-rs" => {
                    additional_nav_bar_elements.push(
                        NavBarElement { 
                            display_text: "Crates.io".to_string(), 
                            href: "https://crates.io/crates/tree_iterators_rs".to_string() 
                        }
                    )   
                }
                "json-formatter" => {
                    additional_nav_bar_elements.push(
                        NavBarElement { 
                            display_text: "Crates.io".to_string(), 
                            href: "https://crates.io/crates/toy-json-formatter".to_string() 
                        }
                    )                       
                }
                _ => {}
            }

            html.push_str(&create_nav_bar(Some(additional_nav_bar_elements)));
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

async fn blog(State(state): State<AppState>) -> Html<String> {
    match github::get_blog_posts(&state).await {
        None => Html(ERROR_RESPONSE.to_string()),
        Some(data) => {
            let mut html = create_html_page(true);
            html.push_str("<body onLoad='onLoad()'>"); {
                html.push_str(&create_nav_bar(None));

                html.push_str("<ul style='display: grid;column-count: 2;column-gap: 20px;row-gap: 20px; margin-right: 30px'>");
                
                for (i, blog_post) in data.iter().enumerate() {
                    html.push_str(&generate_blog_card(i, blog_post));
                }

                html.push_str("</ul>");
            }
            html.push_str("</body>");
            Html(html)
        }
    }
}

async fn blog_post(State(state): State<AppState>, Path(blog): Path<String>) -> Result<Html<String>, StatusCode> {
    match github::get_blog_post(&state, &blog).await {
        None => Err(StatusCode::NOT_FOUND),
        Some(blog_post) => {
            let mut html = create_html_page(false);
            html.push_str("<body onLoad='onLoad()'>");
            html.push_str(&create_nav_bar(None));
            html.push_str("<div>"); {
                html.push_str(&parse_md_to_html(&blog_post.content));
            }
            html.push_str("</div>");
            html.push_str("</body>");
            Ok(Html(html))
        }
    }
}

async fn format_json(json: Form<JsonFormData>) -> Html<String> {
    let mut result = "<textarea style='height: 100%; width: 100%;'>".to_string();
    let (formatted, errs) = toy_json_formatter::format(&json.0.json);
    result.push_str(&formatted);

    if let Some(errs) = errs {
        result.push('\n');
        result.push_str("Errors:\n");
        for err in errs {
            result.push_str(&format!("{}", err));
            result.push('\n');
        }
    }
    result.push_str("</textarea>");
    Html(result)
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

fn create_nav_bar(additional_elements: Option<Vec<NavBarElement>>) -> String {
    let mut html = String::new();
    html.push_str("<nav id='navbar'>"); {
        html.push_str("<ul id='navbar_list' style='list-style: none; display: flex; flex-direction: row; justify-content: left; margin: 0px; padding: 0px;'>"); {
            let buttons = [
                NavBarElement { 
                    display_text: "Home".to_string(), 
                    href: INDEX_URL.to_string() 
                }, 
                NavBarElement { 
                    display_text: "Projects".to_string(), 
                    href: format!("{}/projects", INDEX_URL) 
                },
                NavBarElement {
                    display_text: "Blog".to_string(),
                    href: format!("{}/blog", INDEX_URL)
                }
            ].into_iter()
                .chain(
                    additional_elements.into_iter()
                        .flat_map(|opt| opt)
                );
            
            for element in buttons {
                html.push_str("<li>"); {
                    html.push_str("<a href='");
                    html.push_str(&element.href);
                    html.push_str("'>");
                    html.push_str(&element.display_text);
                    html.push_str("</a>");
                }
            }
        }
        html.push_str("</ul>");
    }
    html.push_str("</nav>");
    html
}

fn generate_repo_card(index: usize, repo: &Repo) -> String {
    let mut html = String::new();
    html.push_str(&format!("<li style='grid-row: {}; grid-column: {}'>", index / 2 + 1, index % 2 + 1)); {
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
    html
}

fn generate_blog_card(index: usize, blog_post: &BlogPost) -> String {
    let mut html = String::new();
    html.push_str(&format!("<li style='grid-row: {}; grid-column: {}'>", index / 2 + 1, index % 2 + 1)); {
        html.push_str("<h2>"); {
            html.push_str(&format!("<a href='{}/blog/{}'>", INDEX_URL, get_url_safe_name(&blog_post.name))); {
                html.push_str(&blog_post.name);
            }
            html.push_str("</a>");
        }
        html.push_str("</h2>");

        html.push_str("<p>"); {
            html.push_str(&blog_post.description);
        }
        html.push_str("</p>");
    }
    html.push_str("</li>");
    html
}

fn get_url_safe_name(name: &str) -> String {
    name.chars().filter(|char| {
        match char {
            'a'..='z'
            | 'A'..='Z'
            | '0'..='9' => true,
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

struct NavBarElement {
    display_text: String,
    href: String,
}

#[derive(Clone)]
struct AppState {
    db_connection: PgPool
}

impl AppState {
    fn new(pool: PgPool) -> Self {
        Self {
            db_connection: pool,
        }
    }
}

#[derive(Deserialize)]
struct JsonFormData {
    json: String,
}