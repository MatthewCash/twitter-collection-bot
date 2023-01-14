use std::path::Path;

pub async fn get_image_file(name: &str) -> Result<Vec<u8>, std::io::Error> {
    let image_dir = std::env::var("IMAGE_DIR_PATH").unwrap_or_else(|_| "./images/".into());

    let path = Path::new(&image_dir).join(name);

    tokio::fs::read(path).await
}
