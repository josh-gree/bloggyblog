use std::{io::Write, path::Path, sync::Arc};

use cached::proc_macro::cached;
use octocrab::{models::repos::Object, params::repos::Reference};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::AppConfig, index::Index};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ArticleEntry {
    pub title: String,
    pub id: Uuid,
}

impl ArticleEntry {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Article {
    pub content: String,
}

impl Article {
    pub async fn from_github(uuid: Uuid, config: &AppConfig) -> Result<Self, String> {
        let owner = config.gh_owner.clone();
        let repo = config.gh_repo.clone();

        let octocrab = octocrab::instance();
        let repo_obj = octocrab.repos(&owner, &repo);
        let r#ref = repo_obj
            .get_ref(&Reference::Branch("main".to_string()))
            .await
            .map_err(|e| e.to_string())
            .map_err(|e| e.to_string())?;

        let sha = match r#ref.object {
            Object::Commit { sha, url: _ } => sha,
            _ => panic!(),
        };

        let path = format!("articles/{}.md", uuid);
        let url = format!(
            "https://cdn.jsdelivr.net/gh/{}/{}@{}/{}",
            owner, repo, sha, path
        );

        let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;

        if resp.status() == StatusCode::NOT_FOUND {
            Err("jsdeliver 404d".into())
        } else {
            let content = resp.text().await.map_err(|e| e.to_string())?;
            Ok(Article { content })
        }
    }
}

#[cached(time = 86400)]
pub async fn article_from_github_cached(
    uuid: Uuid,
    config: Arc<AppConfig>,
) -> Result<Article, String> {
    Article::from_github(uuid, &config).await
}

#[cached(time = 86400)]
pub async fn article_index_from_github_cached(
    config: Arc<AppConfig>,
    path: String,
) -> Result<Index<ArticleEntry>, String> {
    Index::<ArticleEntry>::from_github(&config, path).await
}

fn generate_base_article_file(path: &Path, index_entry: &ArticleEntry) {
    let mut article_file = std::fs::File::create(path).unwrap();
    article_file
        .write_all(format!("# {}", index_entry.title).as_bytes())
        .unwrap();
}

pub fn create_new_article(config: &AppConfig, title: String) {
    let article_index_path = Path::new(&config.article_index_file_path);
    let index_entry = ArticleEntry::new(title);

    let article_file_path = format!("{}/{}.md", config.article_index_base_path, index_entry.id);
    let article_file_path = Path::new(article_file_path.as_str());
    generate_base_article_file(article_file_path, &index_entry);

    if let Ok(mut index) = Index::from_file(&article_index_path) {
        index.0.push(index_entry);
        index.to_file(&article_index_path).unwrap()
    } else {
        let mut index = Index(vec![]);
        index.0.push(index_entry);
        index.to_file(&article_index_path).unwrap();
    }
}
