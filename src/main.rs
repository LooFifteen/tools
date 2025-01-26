#![feature(duration_constructors)]

use std::{collections::HashMap, sync::Arc};

use axum::extract::{Path, Query, State};
use github::GitHub;
use tokio::sync::Mutex;

mod github;

const DEFAULT_BRANCH: &str = "master";

#[tokio::main]
async fn main() {
    let github = Arc::new(Mutex::new(GitHub::new().unwrap()));
    
    let app = axum::Router::new()
        .route("/", axum::routing::get(get))
        .route("/{branch}", axum::routing::get(get_from_branch))
        .with_state(github);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_latest_commit(branch: &str, github: Arc<Mutex<GitHub>>, parameters: HashMap<String, String>) -> Result<String, ()> {
    let commit = github.lock().await.get_latest_successful_commit(&branch).await.map_err(|_| ())?[..10].to_string();

    let hash = if branch == DEFAULT_BRANCH {
        commit
    } else {
        format!("{}-{}", branch, commit)
    };

    let dependency = format!("net.minestom:minestom-snapshots:{}", hash);

    Ok(if parameters.contains_key("kts") {
        format!(include_str!("templates/build.gradle.kts"), dependency)
    } else {
        dependency
    })
}

async fn get_from_branch(
    Path(branch): Path<String>,
    Query(parameters): Query<HashMap<String, String>>,
    State(github): State<Arc<Mutex<GitHub>>>
) -> Result<String, ()> {
    get_latest_commit(&branch, github, parameters).await
}

async fn get(
    State(github): State<Arc<Mutex<GitHub>>>,
    Query(parameters): Query<HashMap<String, String>>
) -> Result<String, ()> {
    get_latest_commit(DEFAULT_BRANCH, github, parameters).await
}