use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::Bytes;
use cached::proc_macro::cached;
use octocrab::{models::repos::Object, params::repos::Reference};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::AppConfig, index::Index};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ImageEntry {
    pub description: String,
    pub id: Uuid,
    pub ext: Option<String>,
}

impl ImageEntry {
    fn new(description: String, ext: Option<String>) -> Self {
        ImageEntry {
            description,
            id: Uuid::new_v4(),
            ext,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Image {
    pub content: Bytes,
}

impl Image {
    pub async fn from_github(
        uuid: Uuid,
        ext: Option<String>,
        config: &AppConfig,
    ) -> Result<Self, String> {
        println!("Getting Image from GH");
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

        let mut path = format!("images/{}", uuid);

        if let Some(ext) = ext {
            path.push_str(".");
            path.push_str(ext.as_str());
        }
        let url = format!(
            "https://cdn.jsdelivr.net/gh/{}/{}@{}/{}",
            owner, repo, sha, path
        );

        let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        Ok(Image { content: bytes })
    }
}

#[cached(time = 86400)]
pub async fn image_from_github_cached(
    uuid: Uuid,
    ext: Option<String>,
    config: Arc<AppConfig>,
) -> Result<Image, String> {
    Image::from_github(uuid, ext, &config).await
}

pub fn create_new_image(config: &AppConfig, image: PathBuf, description: String) {
    let image_index_path = Path::new(&config.image_index_file_path);

    let ext = image.extension();

    let index_entry = if let Some(ext) = ext {
        ImageEntry::new(description, Some(ext.to_str().unwrap().to_string()))
    } else {
        ImageEntry::new(description, None)
    };

    let mut fname = format!(
        "{}/{}",
        config.image_index_base_path,
        index_entry.id.to_string()
    );
    if let Some(ext) = ext {
        fname.push_str(".");
        fname.push_str(ext.to_str().unwrap_or(""))
    }

    let _ = fs::copy(image, fname);
    if let Ok(mut index) = Index::from_file(&image_index_path) {
        index.0.push(index_entry);
        index.to_file(&image_index_path).unwrap()
    } else {
        let mut index = Index(vec![]);
        index.0.push(index_entry);
        index.to_file(&image_index_path).unwrap();
    }
}

#[cached(time = 86400)]
pub async fn image_index_from_github_cached(
    config: Arc<AppConfig>,
    path: String,
) -> Result<Index<ImageEntry>, String> {
    Index::<ImageEntry>::from_github(&config, path).await
}
