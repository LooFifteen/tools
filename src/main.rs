use std::sync::Arc;

use axum::extract::{Path, State};
use axum_response_cache::CacheLayer;
use github::GitHub;

mod github;

const DEFAULT_BRANCH: &str = "master";

#[tokio::main]
async fn main() {
    let github = Arc::new(github::GitHub::new().unwrap());
    
    let app = axum::Router::new()
        .route("/", axum::routing::get(get))
        .route("/{branch}", axum::routing::get(get_from_branch))
        .layer(CacheLayer::with_lifespan(3600).use_stale_on_failure())
        .with_state(github);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_latest_commit(branch: &str, github: Arc<GitHub>) -> Result<String, ()> {
    let commit = github.get_latest_successful_commit(&branch).await.map_err(|_| ())?[..10].to_string();

    let hash = if branch == DEFAULT_BRANCH {
        commit
    } else {
        format!("{}-{}", branch, commit)
    };

    Ok(format!("net.minestom:minestom-snapshots:{}", hash))
}

async fn get_from_branch(Path(branch): Path<String>, State(github): State<Arc<GitHub>>) -> Result<String, ()> {
    get_latest_commit(&branch, github).await
}

async fn get(State(github): State<Arc<GitHub>>) -> Result<String, ()> {
    get_latest_commit(DEFAULT_BRANCH, github).await
}
