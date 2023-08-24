pub mod cache;
pub mod errors;

use crate::errors::Error;
use codecov::{
    author::Author, branch_detail::BranchDetailAPIResponse, branches::BranchesAPIResponse,
    commits::CommitsAPIResponse, owner::Owner, repos::Repo, Client as CodecovClient,
};

/**
 * Client is a struct wrapping CodecovClient.
 */
pub struct Client {
    codecov_client: CodecovClient,
    cache_client: cache::Client,
}

/**
 * Client is a struct wrapping CodecovClient.
 * pub methods are same as CodecovClient.
 * https://docs.rs/codecov/latest/codecov/struct.Client.html
 */
impl Client {
    pub fn new_from_env() -> Result<Client, Error> {
        let cache_dir = Client::resolve_cache_dir_root();
        Ok(Client {
            codecov_client: CodecovClient::new_from_env()?,
            cache_client: cache::Client::new(cache_dir, "data.json".to_string()),
        })
    }

    fn resolve_cache_dir_root() -> std::path::PathBuf {
        match std::env::var("CODECOV_CACHE_DIR") {
            Ok(path) => std::path::PathBuf::from(path),
            Err(_) => Client::default_cache_dir_root(),
        }
    }

    fn default_cache_dir_root() -> std::path::PathBuf {
        let Some(mut path) = dirs::cache_dir() else {
            panic!("Unsupported platform");
        };
        path.push("rust-codecov-cache");
        path
    }

    pub fn new(token: String, cache_dir: std::path::PathBuf) -> Client {
        Client {
            codecov_client: CodecovClient::new(token),
            cache_client: cache::Client::new(cache_dir, "data.json".to_string()),
        }
    }

    /**
     * get_all_repos returns a list of all repos for a given owner.
     * /repos endpoint returns a list of repos for a given owner with pagination.
     * This function will make multiple requests to get all repos.
     */
    pub fn get_all_repos(&self, owner: &Owner) -> Result<Vec<Repo>, Error> {
        Ok(self.codecov_client.get_all_repos(owner)?)
    }

    /**
     * get_commits returns a list of commits for a given author.
     * https://docs.codecov.com/reference/repos_commits_list
     */
    pub fn get_commits(&self, author: &Author) -> Result<CommitsAPIResponse, Error> {
        Ok(self.codecov_client.get_commits(author)?)
    }

    /**
     * get_branches returns a list of branches for a given author.
     * https://docs.codecov.com/reference/repos_branches_list
     */
    pub fn get_branches(&self, author: &Author) -> Result<BranchesAPIResponse, Error> {
        Ok(self.codecov_client.get_branches(author)?)
    }

    /**
     * get_branch_detail returns a branch detail for a given author and branch name.
     * https://docs.codecov.com/reference/repos_branches_retrieve
     */
    pub fn get_branch_detail(
        &self,
        author: &Author,
        branch_name: &str,
    ) -> Result<BranchDetailAPIResponse, Error> {
        Ok(self.codecov_client.get_branch_detail(author, branch_name)?)
    }

    /**
     * get_branch_detail returns a branch detail for a given author and branch name.
     * https://docs.codecov.com/reference/repos_branches_retrieve
     */
    pub fn get_branch_detail_with_commit_id(
        &self,
        author: &Author,
        branch_name: &str,
        commit_id: &str,
    ) -> Result<BranchDetailAPIResponse, Error> {
        let cache_key = &[
            &author.service,
            &author.username,
            &author.name,
            branch_name,
            commit_id,
        ];
        // Use cache if exists
        if let Ok(data) = self.cache_client.load(cache_key) {
            if let Ok(value) = serde_json::from_slice(&data) {
                if let Ok(branch_detail) = serde_json::from_value(value) {
                    return Ok(branch_detail);
                }
            }
        }
        // If cache does not exist, fetch from Codecov API
        let retrieved = self.codecov_client.get_branch_detail(author, branch_name)?;
        // Save to cache
        if let BranchDetailAPIResponse::Success(detail) = &retrieved {
            if let Ok(data) = serde_json::to_vec(&detail) {
                let cache_key = &[
                    &author.service,
                    &author.username,
                    &author.name,
                    branch_name,
                    &detail.head_commit.commitid,
                ];
                if let Err(err) = self.cache_client.save(cache_key, &data) {
                    println!("Failed to save cache: {:?}", err);
                }
            }
            return Ok(retrieved);
        }
        Ok(retrieved)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_and_cache() {
        let client = Client::new_from_env().unwrap(); // Read CODECOV_OWNER_TOKEN from environment variable
        let owner = Owner::new("github", "kitsuyui");
        let author = owner.new_author("rust-codecov");

        let detail = client.get_branch_detail(&author, "main").unwrap();

        let BranchDetailAPIResponse::Success(detail) = detail else {
            panic!("Unexpected response");
        };
        let commit_id = detail.head_commit.commitid.to_string();

        client
            .get_branch_detail_with_commit_id(&author, "main", &commit_id)
            .unwrap();
        assert!(client.cache_client.has(&[
            "github",
            "kitsuyui",
            "rust-codecov",
            "main",
            &commit_id
        ]));

        // Check cache dir exists
        let mut cache_dir = dirs::cache_dir().unwrap();
        cache_dir.extend(&[
            "rust-codecov-cache",
            "github",
            "kitsuyui",
            "rust-codecov",
            "main",
            &commit_id,
        ]);
        assert!(cache_dir.exists());
    }
}
