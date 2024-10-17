use std::env;
use std::fs::{create_dir_all, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub fn extframework_dir() -> PathBuf {
    let path = if cfg!(target_os = "windows") {
        PathBuf::from(env::var("APP_DATA").unwrap())
    } else if cfg!(target_os = "macos") {
        home::home_dir().unwrap()
            .join("Library")
            .join("Application Support")
    } else {
        home::home_dir().unwrap()
    };

    return path
        .join(".minecraft")
        .join(".extframework");
}

fn client_url(version: String) -> String {
    format!("https://maven.extframework.dev/releases/dev/extframework/client/{version}/client-{version}-all.jar", version = version)
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

    // if !response.status().is_success() {
    //     return Err(reqwest::Error::new  )
    // }

    create_dir_all(path.parent().unwrap()).expect("Failed to create client dir path");
    let mut client_file = File::create(path).expect("Failed to open client.jar file");
    let mut content = Cursor::new(response.bytes().await?);

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