use anyhow::Result;
use std::{env, path::Path};
use tokio::fs;

pub async fn get_image_file(name: &str) -> Result<Vec<u8>> {
    let image_dir = env::var("IMAGE_DIR_PATH").unwrap_or_else(|_| "./images/".into());

    Ok(fs::read(Path::new(&image_dir).join(name)).await?)
}
