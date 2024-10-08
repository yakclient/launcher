use std::fs::{create_dir_all, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub fn extframework_dir() -> PathBuf {
    home::home_dir().unwrap()
        .join(".extframework")
}

fn client_url(version: String) -> String {
   format!("https://static.extframework.dev/client/yak-client-{}.jar", version)
}

pub async fn get_client(version: String) -> reqwest::Result<PathBuf> {
    let path = extframework_dir()
        .join(format!("yak-client-{}.jar", version));

    if !Path::new(path.as_os_str()).exists() {
        println!("Downloading client");
        download_client(version, &path).await?
    }
    Ok(path)
}

async fn download_client(
    version: String,
    path: &PathBuf,
) -> reqwest::Result<()> {
    let client_url = client_url(version);
    let response = reqwest::get(client_url).await?;

    create_dir_all(path.parent().unwrap()).expect("Failed to create client dir path");
    let mut client_file = File::create(path).expect("Failed to open client.jar file");
    let mut content =  Cursor::new(response.bytes().await?);

    std::io::copy(&mut content, &mut client_file).expect("Failed to copy client.jar file");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::launch::client::get_client;

    #[tokio::test]
    async fn test_client_download() {
        get_client("1.0-SNAPSHOT".to_string()).await.unwrap();
    }
}