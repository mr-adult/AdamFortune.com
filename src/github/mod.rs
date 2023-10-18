use std::{sync::RwLock, time::Duration, fs::OpenOptions, io::{Write, Read}};

use base64::Engine;
use chrono::{Utc, DateTime};
use reqwest::{ClientBuilder, Client};
use serde_derive::{Deserialize, Serialize};

const URL: &'static str = "https://api.github.com/";
const USERNAME: &'static str = "mr-adult";
const CACHE_FILE_PATH: &'static str = "./src/github/cache.json";

static UPDATE_IN_PROGRESS: RwLock<bool> = RwLock::new(false);
lazy_static!{
    static ref CACHE: RwLock<GithubData> = RwLock::new(GithubData::default());
}

pub (crate) async fn get_data() -> Result<GithubData, ()> {
    let mut out_of_date = false;
    {
        let index = CACHE.read().unwrap();
        if index.last_updated < (Utc::now() - chrono::Duration::hours(1)) {
            out_of_date = true;
        }
    }

    if out_of_date {
        // make sure another thread isn't already querying 
        // github before we fire off some queries
        if !*UPDATE_IN_PROGRESS.read().unwrap() {
            fetch_github_repos().await?
        }
    }
    Ok(CACHE.read().unwrap().clone())
}

async fn fetch_github_repos() -> Result<(), ()> {
    { *UPDATE_IN_PROGRESS.write().unwrap() = true; }
    let last_updated;
    { last_updated = CACHE.read().unwrap().last_updated; }

    if let Ok(_) = fetch_data_from_file(last_updated) {
        return Ok(());
    }

    let mut get_all_repos_url = URL.to_owned();
    get_all_repos_url.push_str(&format!("users/{}/repos", USERNAME));

    let timeout = Duration::from_secs(5);
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(crate::ACCEPT_INVALID_CERTS)
        .timeout(timeout)
        .user_agent("adamfortune.com server")
        .build();

    let client = match client {
        Ok(inner) => inner,
        Err(_) => Err(())?,
    };

    let response = client
        .get(&get_all_repos_url)
        .query(&[("username", "mr-adult")])
        .send()
        .await;

    let response = match response {
        Err(_) => Err(())?,
        Ok(inner) => inner,
    };
    
    let json: Result<Vec<Repo>, _> = response
        .json()
        .await;

    let mut repos = match json {
        Err(err) => {
            println!("{:?}", err);
            return Err(());
        }
        Ok(json) => json,
    };

    for repo in repos.iter_mut() {
        if repo.name == "blog-posts" {
            if let Some(posts) = get_all_md_files(repo, &client).await {
                let mut results = Vec::new();
                for post in posts {
                    if post.name == "Home" {
                        CACHE.write().unwrap().home = post;
                    } else {
                        results.push(post);
                    }
                }
                CACHE.write().unwrap().blog_posts = results;
            } else {
                crate::utils::log_error("Failed to load blog posts".to_string());
            }
        } else {
            repo.readme = get_read_me(repo, &client).await;
        }
    }
    // blog-posts is special. Don't show it as an actual repo.
    repos = repos.into_iter().filter(|repo| repo.name != "blog-posts").collect();

    {
        let mut index_write = CACHE.write().unwrap();
        if index_write.last_updated < Utc::now() - chrono::Duration::hours(1) {
            index_write.repos = repos;
            index_write.last_updated = Utc::now();
        }
    }

    { *UPDATE_IN_PROGRESS.write().unwrap() = false; }

    flush_data_to_file();

    return Ok(());
}

async fn get_read_me(repo: &Repo, client: &Client) -> Option<String> {
    get_file_content(repo, client, "README.md").await
}

async fn get_file_content(repo: &Repo, client: &Client, path: &str) -> Option<String> {
    let mut get_repo_content_url = URL.to_owned();
    get_repo_content_url.push_str(&format!("repos/{}/{}/contents/{}", USERNAME, &repo.name, path));

    let response = client
        .get(&get_repo_content_url)
        .query(&[
            ("owner", USERNAME),
            ("repo", &repo.name),
            ("path", path)
        ])
        .send()
        .await;

    let response = match response {
        Err(_) => return None,
        Ok(inner) => inner,
    };
    
    let file_content: Result<Readme, _> = response
        .json()
        .await;
    
    match file_content {
        Err(err) => {
            println!("{:?}", err);
            None
        }
        Ok(file_content) => {
            let engine = base64::engine::general_purpose::GeneralPurpose::new(
                &base64::alphabet::STANDARD, 
                base64::engine::GeneralPurposeConfig::new()
            );
            match engine.decode(file_content.content.replace("\n", "")) {
                Err(err) => {
                    println!("{:?}", err);
                    None
                }
                Ok(str) => {
                    Some(String::from_utf8_lossy(str.as_slice()).to_string())
                }
            }
        }
    }
}

async fn get_all_md_files(repo: &Repo, client: &Client) -> Option<Vec<BlogPost>> {
    let mut get_repo_content_url = URL.to_owned();
    get_repo_content_url.push_str(&format!("repos/{}/{}/contents/", USERNAME, &repo.name));

    let response = client
        .get(&get_repo_content_url)
        .query(&[
            ("owner", USERNAME),
            ("repo", &repo.name),
        ])
        .send()
        .await;

    let response = match response {
        Err(_) => return None,
        Ok(inner) => inner,
    };
    
    let files: Result<Vec<FileMetadata>, _> = response
        .json()
        .await;
    
    match files {
        Err(err) => {
            println!("{:?}", err);
            None
        }
        Ok(files) => {
            let mut file_contents = Vec::new();
            for file in files.into_iter().filter(|file| file.path.ends_with(".md")) {
                let content = get_file_content(repo, client, &file.path).await;
                match content {
                    None => continue,
                    Some(content) => {
                        let mut content_without_description = Vec::new();
                        let mut description = Vec::new();
                        for line in content.lines() {
                            if line.starts_with("///") {
                                description.push(line);
                            } else {
                                content_without_description.push(line);
                            }
                        }

                        file_contents.push(BlogPost { 
                                name: file.name[0..file.name.len() - 3].to_string(), 
                                content: content_without_description.join("\n"),
                                description: description.join(" "),
                            });
                    }
                }
            }
            Some(file_contents)
        }
    }
}

#[cfg(debug_assertions)]
fn fetch_data_from_file(last_updated: DateTime<Utc>) -> Result<(), ()> {
    {
        // github has a rate limit of 60 requests/hour, so use a 
        // file so new debugging deployments don't hit the API
        if last_updated == DateTime::<Utc>::default() {
            if let Ok(mut file) = OpenOptions::new().read(true).write(false).open(CACHE_FILE_PATH) {
                let mut result = String::new();
                if let Ok(_) = file.read_to_string(&mut result) {
                    if let Ok(data) = serde_json::from_str::<GithubData>(&result) {
                        if data.last_updated > Utc::now() - chrono::Duration::hours(1) {
                            *CACHE.write().unwrap() = data;
                            return Ok(());
                        }
                    }
                }
            } 
        }

        return Err(());
    }
}

#[cfg(debug_assertions)]
fn flush_data_to_file() {
    // github has a rate limit of 60 requests/hour, so use a 
    // file so new debugging deployments don't hit the API
    let content_to_flush_to_file;
    {
        let index_read = CACHE.read().unwrap();
        content_to_flush_to_file = serde_json::to_string_pretty(&*index_read).unwrap();
    }

    let mut open_options = OpenOptions::new();
    
    open_options
        .read(false)
        .write(true)
        .truncate(true);

    match open_options.open(CACHE_FILE_PATH) {
        Err(err) => {
            crate::utils::log_error(err);
            match open_options.create(true).open(CACHE_FILE_PATH) {
                Err(err) => crate::utils::log_error(err),
                Ok(mut file) => {
                    if let Err(err) = file.write_all(content_to_flush_to_file.as_bytes()) {
                        crate::utils::log_error(err);
                    }
                }
            }
        }
        Ok(mut file) => {
            if let Err(err) = file.write_all(content_to_flush_to_file.as_bytes()) {
                crate::utils::log_error(err);
            }
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub (crate) struct GithubData {
    last_updated: DateTime<Utc>,
    pub (crate) repos: Vec<Repo>,
    pub (crate) home: BlogPost,
    pub (crate) blog_posts: Vec<BlogPost>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub (crate) struct Repo {
    pub (crate) name: String,
    pub (crate) url: String,
    pub (crate) html_url: String,
    pub (crate) description: String,
    pub (crate) updated_at: DateTime<Utc>,
    pub (crate) readme: Option<String>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub (crate) struct BlogPost {
    pub (crate) name: String,
    pub (crate) description: String,
    pub (crate) content: String,
}

#[derive(Deserialize, Serialize)]
pub (crate) struct FileMetadata {
    sha: String,
    name: String,
    path: String,
}

#[derive(Deserialize, Serialize)]
pub (crate) struct Readme {
    pub (crate) content: String
}