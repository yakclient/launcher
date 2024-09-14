use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

fn extframework_dir() -> PathBuf {
    home::home_dir().unwrap()
        .join(".extframework")
}

// TODO This is wrong
fn client_url(version: String) -> &'static str {
    "https://maven.extframework.dev/snapshots/dev/extframework/client/2.1.1-SNAPSHOT/client-2.1.1-20240908.175507-1-all.jar"
}

pub async fn get_client(version: String) -> reqwest::Result<PathBuf> {
    let path = extframework_dir()
        .join(format!("client-{}.jar", version));

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
    let mut client_file = File::create(path).expect("Failed to open client.jar file");
    let mut content =  Cursor::new(response.bytes().await?);

    std::io::copy(&mut content, &mut client_file).expect("Failed to copy client.jar file");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::launch::client::download_client;

    // #[test]
    // fn test_client_download() {
    //     download_client("doesnt matter".to_string()).unwrap()
    // }
}