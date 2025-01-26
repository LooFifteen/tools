use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;
use ttl_cache::TtlCache;

const URL: &str = "https://api.github.com/repos/Minestom/Minestom/commits";
const CACHE_DURATION: Duration = Duration::from_hours(1);

pub(crate) struct GitHub {
    client: reqwest::Client,
    cache: TtlCache<String, String>,
}

impl GitHub {
    pub(crate) fn new() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent(format!("LooFifteen/tools v{}", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            client,
            cache: TtlCache::new(100), // arbitrary value
        })
    }

    async fn get_latest_commit(&self, branch: &str) -> Result<Commits, reqwest::Error> {
        let url = format!("{}/{}", URL, branch);
        let response = self.client.get(url).send().await?;
        Ok(response.json().await?)
    }

    pub(crate) async fn get_latest_successful_commit(&mut self, branch: &str) -> Result<String, LatestCommitError> {
        // if the commit is already in the cache, return it
        if let Some(commit) = self.cache.get(branch) {
            return Ok(commit.clone());
        }

        let commits = self.get_latest_commit(branch).await?;
        if self.check_commit_run(&commits.sha).await? {
            self.cache.insert(branch.to_string(), commits.sha.clone(), CACHE_DURATION);
            return Ok(commits.sha)
        }

        for parent in commits.parents.into_iter() {
            if self.check_commit_run(&parent.sha).await? {
                self.cache.insert(branch.to_string(), parent.sha.clone(), CACHE_DURATION);
                return Ok(parent.sha);
            }
        }
        
        self.cache.insert(branch.to_string(), "not-found".to_string(), CACHE_DURATION);
        Err(LatestCommitError::NoSuccessfulCommit)
    }

    async fn check_commit_run(&self, sha: &String) -> Result<bool, reqwest::Error> {
        let url = format!("{}/{}/check-runs", URL, sha);
        let response = self.client.get(url).send().await?;
        let check_runs: CheckRuns = response.json().await?;

        for check_run in check_runs.check_runs {
            if check_run.conclusion != "success" {
                return Ok(false);
            }
        }

        Ok(true)
    }

}

#[derive(Debug, Error)]
pub(crate) enum LatestCommitError {
    #[error("request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("no successful commit found")]
    NoSuccessfulCommit,
}

#[derive(Deserialize)]
struct Commits {
    sha: String,
    parents: Vec<CommitRef>,
}

#[derive(Deserialize)]
struct CommitRef {
    sha: String,
}

#[derive(Deserialize)]
struct CheckRuns {
    check_runs: Vec<CheckRun>,
}

#[derive(Deserialize)]
struct CheckRun {
    conclusion: String,
}