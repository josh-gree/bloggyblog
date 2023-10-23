use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    ptr,
    sync::Arc,
};

use bytes::Bytes;
use cached::proc_macro::cached;
use exif::{In, Reader, Tag};
use image::DynamicImage;
use libwebp_sys::WebPEncodeRGB;
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

pub fn create_new_image(config: &AppConfig, path: PathBuf, description: String) {
    // create an ImageEntry -> will always be .webp ext
    let index_entry = ImageEntry::new(description, Some("webp".into()));
    if let Some(ext) = path.extension() {
        let fname = format!(
            "{}/{}.webp",
            config.image_index_base_path,
            index_entry.id.to_string()
        );
        if ext != "webp" {
            // convert it and store in right place
            let img = image::open(&path).expect("Should be able to open image");
            let img = correct_orientation(img, &path).expect("Should be able to convert");
            let img_webp_raw = convert_to_webp(img, 0.75);

            let mut file = File::create(fname).expect("should be able to create file");
            file.write_all(&img_webp_raw)
                .expect("should be able to write image data");
        } else {
            // just store it in right place
            let _ = fs::copy(path, fname);
        }
    } else {
        // there is no extension so lets actually panic?
        panic!("There needs to be an extension on the image!!")
    }
    // Add ImageEntry to Index
    let image_index_path = Path::new(&config.image_index_file_path);
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

pub fn convert_to_webp(img: DynamicImage, quality: f32) -> Vec<u8> {
    let img = img.to_rgb8();
    let h = img.height();
    let w = img.width();
    let data = img.into_raw();

    let out = unsafe {
        let mut output_data = ptr::null_mut();
        let stride = w as i32 * 3;
        let len = WebPEncodeRGB(
            data.as_ptr(),
            w as i32,
            h as i32,
            stride,
            quality,
            &mut output_data,
        );
        std::slice::from_raw_parts(output_data, len as usize).into()
    };
    out
}

pub fn correct_orientation(img: DynamicImage, img_path: &Path) -> Result<DynamicImage, String> {
    let file = std::fs::File::open(img_path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = exifreader.read_from_container(&mut bufreader).unwrap();

    let orientation_field = exif_data.get_field(Tag::Orientation, In::PRIMARY);
    if let Some(field) = orientation_field {
        if let Some(orientation) = field.value.get_uint(0) {
            return Ok(match orientation {
                1 => img,
                2 => img.fliph(),
                3 => img.rotate180(),
                4 => img.rotate180().fliph(),
                5 => img.rotate90().fliph(),
                6 => img.rotate90(),
                7 => img.rotate270().fliph(),
                8 => img.rotate270(),
                _ => img, // Keep the original image if orientation value is unknown
            });
        }
    }

    Ok(img)
}
