use octocrab::{models::repos::Object, params::repos::Reference};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::Write, path::Path};

use crate::config::AppConfig;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Index<T>(pub Vec<T>)
where
    T: Serialize;

impl<'de, T> Index<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn to_file(&self, path: &Path) -> Result<(), String> {
        let index = serde_json::to_string(self).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::create(path).map_err(|e| e.to_string())?;
        f.write_all(index.as_bytes()).map_err(|e| e.to_string())
    }
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let index = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_string(index)
    }
    pub fn from_string(index: String) -> Result<Self, String> {
        serde_json::from_str(index.as_str()).map_err(|e| e.to_string())
    }
    pub async fn from_github(config: &AppConfig, path: String) -> Result<Self, String> {
        println!("Getting Index from GH");
        let owner = config.gh_owner.clone();
        let repo = config.gh_repo.clone();

        let octocrab = octocrab::instance();
        let repo_obj = octocrab.repos(&owner, &repo);
        let r#ref = repo_obj
            .get_ref(&Reference::Branch("main".to_string()))
            .await
            .map_err(|e| e.to_string())?;

        let sha = match r#ref.object {
            Object::Commit { sha, url: _ } => sha,
            _ => panic!(),
        };

        let url = format!(
            "https://cdn.jsdelivr.net/gh/{}/{}@{}/{}",
            owner, repo, sha, path
        );

        println!("{:?}", url);

        let index = reqwest::get(url)
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Self::from_string(index)
    }
}
