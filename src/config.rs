use config::Config;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct AppConfig {
    pub article_index_file_path: String,
    pub article_index_base_path: String,
    pub image_index_file_path: String,
    pub image_index_base_path: String,
    pub gh_owner: String,
    pub gh_repo: String,
    pub gh_article_index_file_path: String,
}

pub fn config_setup() -> AppConfig {
    let config = Config::builder()
        .set_default("article_index_file_path", "./articles/index.json")
        .unwrap()
        .set_default("article_index_base_path", "./articles")
        .unwrap()
        .set_default("image_index_file_path", "./images/index.json")
        .unwrap()
        .set_default("image_index_base_path", "./images")
        .unwrap()
        .set_default("gh_owner", "josh-gree")
        .unwrap()
        .set_default("gh_repo", "bloggyblog")
        .unwrap()
        .set_default("gh_article_index_file_path", "articles/index.json")
        .unwrap()
        .build()
        .unwrap();

    config.try_deserialize().unwrap()
}
