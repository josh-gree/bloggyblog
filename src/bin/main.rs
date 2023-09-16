use std::path::PathBuf;

use bloggyblog::{
    article::create_new_article, config::config_setup, image::create_new_image, server::serve,
};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use tokio::runtime::Runtime;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct AppConfig {
    article_index_file_path: String,
    article_index_base_path: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    NewArticle { title: String },
    NewImage { image: PathBuf, description: String },
    Serve,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config_setup();

    // println!("{:?}", config); # TODO: Proper logging

    let args = Args::parse();

    match args.command {
        Some(Commands::NewArticle { title }) => create_new_article(&config, title),
        Some(Commands::NewImage { description, image }) => {
            create_new_image(&config, image, description)
        }
        Some(Commands::Serve) => {
            let rt = Runtime::new()?;
            rt.block_on(serve(config.clone()))
        }
        _ => {}
    }

    Ok(())
}
