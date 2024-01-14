use axum::{
    extract::{DefaultBodyLimit, Path, State},
    response::Html,
    routing::{get, post},
    Router, Json,
};
use github::{BlogPost, Repo};
use pulldown_cmark::{html, Options, Parser};
use serde_derive::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use http::{Method, StatusCode};
use tower_http::{cors::{Any, CorsLayer}, services::{ServeDir, ServeFile}};

mod github;
mod utils;

/// this flag is to set up debugging instances to allow self-signed certificates.
#[cfg(not(debug_assertions))]
pub(crate) const ACCEPT_INVALID_CERTS: bool = false;
#[cfg(debug_assertions)]
pub(crate) const ACCEPT_INVALID_CERTS: bool = true;

#[tokio::main]
async fn main() {
    // First, parse the .env file for our environment setup.
    dotenvy::dotenv().ok();

    // We create a single connection pool for SQLx that's shared across the whole application.
    // This saves us from opening a new connection for every API call, which is wasteful.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
        // We should leave some connections available for manual access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(10)
        .connect(&database_url)
        .await
        .unwrap_or_else(|err| panic!("Could not connect to dabase_url. Error: \n{}", err));

    // Run any SQL migrations to get the DB into the correct state
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|err| panic!("Failed to migrate the database. Error: \n{}", err));

    let mut current_dir = std::env::current_dir().expect("Failed to detect current directory.");
    println!("{}", current_dir.to_string_lossy());
    current_dir.push("dist");
    let mut index = current_dir.clone();
    index.push("index.html");

    // Set up the routes for our application
    let app = Router::new()
        .nest_service("/", ServeDir::new(&current_dir).fallback(ServeFile::new(&index)))
        .route("/home", get(home))
        .route("/projects_json", get(projects))
        .route("/projects_json/:project", get(project))
        .route("/blog_json", get(blog))
        .route("/blog_json/:blog", get(blog_post))
        .route("/parsejson", post(parse_json))
        .route("/formatjson", post(format_json))
        // Attach our connection pool to every endpoint so the endpoints can query the DB.
        .with_state(AppState::new(pool))
        .layer(
            // Add CORS so it doesn't block our requests from the browser
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        )
        .layer(DefaultBodyLimit::max(20_000_000_000)); // raise the limit to 20 GB

    // Bind to port 8080
    let listener = tokio::net::TcpListener::bind("[::]:8080")
        .await
        .unwrap_or_else(|err| panic!("Failed to initialize TCP listener. Error: \n{}", err));

    // Serve is an infinite async function, so we have to report that we're listening before awaiting.
    println!("Now listening on port 8080");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|err| panic!("Failed to start app. Error: \n{}", err));
}

async fn home(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    match github::get_home(state.clone()).await {
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Some(data) => Ok(Html(parse_md_to_html(&data.content))),
    }
}

#[derive(Serialize,Deserialize)]
pub(crate) struct RepoDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) html_url: String,
    pub(crate) description: String,
    pub(crate) readme: Option<String>,
    pub(crate) url_safe_name: String,
    pub(crate) additional_nav_elements: Vec<NavBarElement>,
}

impl From<Repo> for RepoDTO {
    fn from(value: Repo) -> Self {
        Self {
            additional_nav_elements: vec![
                Some(NavBarElement {
                    display_text: "Source Code".to_string(),
                    href: value.html_url.to_string(),
                }),
                match value.name.as_str() {
                    "tree-iterators-rs" => Some(NavBarElement {
                        display_text: "Crates.io".to_string(),
                        href: "https://crates.io/crates/tree_iterators_rs".to_string(),
                    }),
                    "json-formatter" => Some(NavBarElement {
                        display_text: "Crates.io".to_string(),
                        href: "https://crates.io/crates/toy-json-formatter".to_string(),
                    }),
                    _ => None,
                }
            ].into_iter()
            .flat_map(|vec| vec)
            .collect(),
            url_safe_name: get_url_safe_name(&value.name),
            id: value.id,
            name: value.name,
            url: value.url,
            html_url: value.html_url,
            description: value.description,
            readme: if let Some(readme) = value.readme { 
                Some(parse_md_to_html(&readme)) 
            } else { 
                None 
            },
        }
    }
}

async fn projects(State(state): State<AppState>) -> Result<Json<Vec<RepoDTO>>, StatusCode> {
    match github::get_repos(state.clone()).await {
        None => Err(StatusCode::NOT_FOUND),
        Some(data) => {
            Ok(Json(data.into_iter().map(|repo| repo.into()).collect()))
        }
    }
}

async fn project(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Json<RepoDTO>, StatusCode> {
    match github::get_repo(&state.clone(), &project).await {
        None => Err(StatusCode::NOT_FOUND),
        Some(repo) => Ok(Json(repo.into()))
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BlogPostDTO {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) alphanumeric_name: String,
    pub(crate) sha: String,
    pub(crate) description: String,
    pub(crate) content: String,
    pub(crate) url_safe_name: String,
}

impl From<BlogPost> for BlogPostDTO {
    fn from(value: BlogPost) -> Self {
        BlogPostDTO {
            url_safe_name: get_url_safe_name(&value.name),
            id: value.id,
            name: value.name,
            alphanumeric_name: value.alphanumeric_name,
            sha: value.sha,
            description: value.description,
            content: parse_md_to_html(&value.content),
        }
    }
}

async fn blog(State(state): State<AppState>) -> Result<Json<Vec<BlogPostDTO>>, StatusCode> {
    match github::get_blog_posts(state.clone()).await {
        None => Err(StatusCode::NOT_FOUND),
        Some(mut data) => {
            data.sort_by(|post1, post2| post2.description.cmp(&post1.description));
            Ok(Json(data.into_iter().map(|post| post.into()).collect()))
        }
    }
}

async fn blog_post(
    State(state): State<AppState>,
    Path(blog): Path<String>,
) -> Result<Json<BlogPostDTO>, StatusCode> {
    match github::get_blog_post(&state.clone(), &blog).await {
        None => Err(StatusCode::NOT_FOUND),
        Some(blog_post) => Ok(Json(blog_post.into()))
    }
}

async fn parse_json(json: Json<JsonFormData>) -> Json<String> {
    let jsons;
    match json.0.format {
        JsonFormat::JsonLines => {
            jsons = json.0.json.lines().collect();
        }
        JsonFormat::JsonStandard => {
            jsons = vec![&json.0.json[..]];
        }
    }

    let results = jsons.into_iter().map(|json| {
        let result = toy_json_formatter::parse(json);
        match result {
            Ok(inner) => Ok(inner),
            Err(_) => {
                let result = toy_json_formatter::format(json);

                let result_with_err_strings = (
                    result.0,
                    result.1.into_iter().map(|err| format!("{}", err)).collect::<Vec<_>>()
                );

                Err(result_with_err_strings)
            }
        }

    })
    .collect::<Vec<_>>();

    let json_string = serde_json::to_string(&results).expect("JSON value to always be JSON serializable.");
    Json(json_string)
}

async fn format_json(json: Json<String>) -> Json<String> {
    Json(toy_json_formatter::format(json.as_str()).0)
}

fn get_url_safe_name(name: &str) -> String {
    name.chars()
        .filter(|char| match char {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
        .collect()
}

fn parse_md_to_html(md: &str) -> String {
    let parser = Parser::new_ext(&md, Options::empty());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[derive(Serialize, Deserialize)]
struct NavBarElement {
    display_text: String,
    href: String,
}

#[derive(Clone)]
struct AppState {
    db_connection: PgPool,
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
