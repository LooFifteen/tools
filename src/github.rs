use serde::Deserialize;
use thiserror::Error;

const URL: &str = "https://api.github.com/repos/Minestom/Minestom/commits";

pub(crate) struct GitHub {
    client: reqwest::Client,
}

impl GitHub {
    pub(crate) fn new() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent("LooFifteen/minestom-ver v0.1.0")
            .build()?;
        Ok(Self {
            client
        })
    }

    async fn get_latest_commit(&self, branch: &str) -> Result<Commits, reqwest::Error> {
        let url = format!("{}/{}", URL, branch);
        let response = self.client.get(url).send().await?;
        Ok(response.json().await?)
    }

    pub(crate) async fn get_latest_successful_commit(&self, branch: &str) -> Result<String, LatestCommitError> {
        let commits = self.get_latest_commit(branch).await?;
        if self.check_commit_run(&commits.sha).await? {
            return Ok(commits.sha)
        }

        for parent in commits.parents.into_iter() {
            if self.check_commit_run(&parent.sha).await? {
                return Ok(parent.sha);
            }
        }
        
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