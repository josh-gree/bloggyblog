use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use axum_extra::either::Either;
use cached::Cached;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    article::{
        article_from_github_cached, article_index_from_github_cached, Article, ArticleEntry,
        ARTICLE_FROM_GITHUB_CACHED, ARTICLE_INDEX_FROM_GITHUB_CACHED,
    },
    config::AppConfig,
    image::{
        image_from_github_cached, image_index_from_github_cached, Image, ImageEntry,
        IMAGE_FROM_GITHUB_CACHED, IMAGE_INDEX_FROM_GITHUB_CACHED,
    },
    index::Index,
};

pub async fn serve(config: AppConfig) {
    let state = Arc::new(config);
    let app = Router::new()
        .route("/", get(home))
        .route("/articles", get(articles))
        .route("/article/:uuid", get(article))
        .route("/images", get(images))
        .route("/image/:uuid", get(image))
        .route("/image/raw/:uuid", get(image_raw))
        .route("/cache-clear", get(cache_clear))
        .with_state(state);

    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct QueryParams {
    no_cache: Option<bool>,
}

async fn articles(
    Query(params): Query<QueryParams>,
    State(config): State<Arc<AppConfig>>,
) -> impl IntoResponse {
    let path = config.article_index_file_path.clone();
    let index = match params.no_cache {
        Some(no_cache) if no_cache => {
            if let Ok(index) = Index::<ArticleEntry>::from_github(&config, path).await {
                Some(index)
            } else {
                None
            }
        }
        None | Some(_) => {
            if let Ok(index) = article_index_from_github_cached(config, path).await {
                Some(index)
            } else {
                None
            }
        }
    };

    if let Some(index) = index {
        Either::E1(ArticlesTemplate { index })
    } else {
        Either::E2(ArticleTemplate {
            content: "Unable to load index".to_string(),
        })
    }
}

async fn article(
    Query(params): Query<QueryParams>,
    State(config): State<Arc<AppConfig>>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    // println!("Article endpoint has been hit!"); # TODO: proper logging
    let article = match params.no_cache {
        Some(no_cache) if no_cache => {
            if let Ok(article) = Article::from_github(uuid, &config).await {
                Some(article)
            } else {
                None
            }
        }
        None | Some(_) => {
            if let Ok(article) = article_from_github_cached(uuid, config).await {
                Some(article)
            } else {
                None
            }
        }
    };
    if let Some(article) = article {
        ArticleTemplate {
            content: article.content,
        }
    } else {
        ArticleTemplate {
            content: "Article Not found".to_string(),
        }
    }
}

async fn images(
    Query(params): Query<QueryParams>,
    State(config): State<Arc<AppConfig>>,
) -> impl IntoResponse {
    let path = config.image_index_file_path.clone();
    let index = match params.no_cache {
        Some(no_cache) if no_cache => {
            if let Ok(index) = Index::<ImageEntry>::from_github(&config, path).await {
                Some(index)
            } else {
                None
            }
        }
        None | Some(_) => {
            if let Ok(index) = image_index_from_github_cached(config, path).await {
                Some(index)
            } else {
                None
            }
        }
    };

    if let Some(index) = index {
        Either::E1(ImagesTemplate { index })
    } else {
        Either::E2(ArticleTemplate {
            content: "Unable to load index".to_string(),
        })
    }
}

async fn image_raw(
    Query(params): Query<QueryParams>,
    State(config): State<Arc<AppConfig>>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    // println!("Article endpoint has been hit!"); # TODO: proper logging
    let path = config.image_index_file_path.clone();
    let index = match params.no_cache {
        Some(no_cache) if no_cache => Index::<ImageEntry>::from_github(&config, path)
            .await
            .unwrap(),
        None | Some(_) => image_index_from_github_cached(config.clone(), path)
            .await
            .unwrap(),
    };

    let img_entry: Vec<_> = index.0.iter().filter(|im| im.id == uuid).collect();
    let img_entry = img_entry.first().unwrap();

    let ext = img_entry.ext.clone();
    let image = match params.no_cache {
        Some(no_cache) if no_cache => {
            if let Ok(image) = Image::from_github(uuid, ext.clone(), &config).await {
                Some(image)
            } else {
                None
            }
        }
        None | Some(_) => {
            if let Ok(image) = image_from_github_cached(uuid, ext.clone(), config).await {
                Some(image)
            } else {
                None
            }
        }
    };
    if let Some(image) = image {
        let mime = mime_guess::from_ext(ext.unwrap().as_str())
            .first_or_octet_stream()
            .essence_str()
            .to_owned();
        Either::E1(([(CONTENT_TYPE, mime)], image.content))
    } else {
        Either::E2("Article Not found".to_string())
    }
}

async fn image(Path(uuid): Path<Uuid>, Query(params): Query<QueryParams>) -> impl IntoResponse {
    let img = match params.no_cache {
        Some(no_cache) if no_cache => {
            format!(
                "<img src=\"/image/raw/{}?no_cache=true\" width=\"500\" height=\"600\">",
                uuid
            )
        }
        None | Some(_) => {
            format!(
                "<img src=\"/image/raw/{}\" width=\"500\" height=\"600\">",
                uuid
            )
        }
    };

    Html(img)
}

async fn home() -> impl IntoResponse {
    HomeTemplate {}
}

async fn cache_clear() -> impl IntoResponse {
    let mut cache = ARTICLE_FROM_GITHUB_CACHED.lock().await;
    cache.cache_clear();

    let mut cache = ARTICLE_INDEX_FROM_GITHUB_CACHED.lock().await;
    cache.cache_clear();

    let mut cache = IMAGE_FROM_GITHUB_CACHED.lock().await;
    cache.cache_clear();

    let mut cache = IMAGE_INDEX_FROM_GITHUB_CACHED.lock().await;
    cache.cache_clear();

    "Ok"
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    content: String,
}

#[derive(Template)]
#[template(path = "articles.html")]
struct ArticlesTemplate {
    index: Index<ArticleEntry>,
}

#[derive(Template)]
#[template(path = "images.html")]
struct ImagesTemplate {
    index: Index<ImageEntry>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}
