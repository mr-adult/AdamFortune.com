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

const ERROR_RESPONSE: &'static str = "Failed to reach database.";

const ALL_PAGES_CSS: &'static str = include_str!("./index.css");

const CONTENT_LIST_CSS: &'static str = include_str!("./content_list.css");

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

                html.push_str("<ul style='display: grid; column-count: 2; column-gap: 20px; row-gap: 20px; padding: 0px; word-break: break-word'>");
                
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
                                <label for="type">JSON Type:</label><br/>
                                <input type="radio" id="jsonStandard" name="format" value="JsonStandard" checked>
                                <label for="jsonStandard">Standard JSON</label><br>
                                <input type="radio" id="jsonLines" name="format" value="JsonLines">
                                <label for="jsonLines">Json Lines Format</label><br>  
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
        Some(mut data) => {
            data.sort_by(|post1, post2| post2.description.cmp(&post1.description));
            let mut html = create_html_page(true);
            html.push_str("<body onLoad='onLoad()'>"); {
                html.push_str(&create_nav_bar(None));

                html.push_str("<ul style='display: grid; column-count: 2; column-gap: 20px; row-gap: 20px; padding: 0px; word-break: break-word;'>");
                
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
    let mut result = String::new();

    let jsons;
    match json.0.format {
        JsonFormat::JsonLines => {
            jsons = json.0.json.lines().collect();
        }
        JsonFormat::JsonStandard => {
            jsons = vec![&json.0.json[..]];
        }
    }
    for json in jsons {
        result.push_str("<textarea style='height: 50%; width: 100%;'>");
        let (formatted, errs) = toy_json_formatter::format(json);
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
    }
    Html(result)
}

/// Creates an HTML page, adding the <head> tag that is needed. 
/// Callers should add the <body> tag and all inner content
fn create_html_page(is_content_list: bool) -> String {
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str("<head>"); {
        html.push_str(r#"
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/atom-one-dark.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>

<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/python.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/csharp.min.js"></script>
"#);
        html.push_str("<script>"); {
            html.push_str(include_str!("./onload.js"))
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
        html.push_str("<ul id='navbar_list' style='list-style: none; display: flex; flex-direction: row; justify-content: space-around; margin: 0px; padding: 0px;'>"); {
            let buttons = [
                NavBarElement { 
                    display_text: "Home".to_string(), 
                    href: "/".to_string()
                }, 
                NavBarElement { 
                    display_text: "Projects".to_string(), 
                    href: "/projects".to_string() 
                },
                NavBarElement {
                    display_text: "Blog".to_string(),
                    href: "/blog".to_string()
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
    html.push_str(&format!("<li style='grid-row: {}; grid-column: {}'>", index + 1, 1)); {
        html.push_str("<h2>"); {
            html.push_str(&format!("<a href='/projects/{}'>", get_url_safe_name(&repo.name))); {
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
    html.push_str(&format!("<li style='grid-row: {}; grid-column: {}'>", index + 1, 1)); {
        html.push_str("<h2>"); {
            html.push_str(&format!("<a href='/blog/{}'>", get_url_safe_name(&blog_post.name))); {
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
    format: JsonFormat,
    json: String,
}

#[derive(Deserialize)]
enum JsonFormat {
    JsonStandard,
    JsonLines,
}