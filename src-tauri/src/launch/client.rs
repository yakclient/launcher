use crate::extframework_dir;
use crate::launch::ClientError;
use crate::launch::ClientError::{IoError, NetworkError};
use std::cmp::min;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{env, io};

fn client_url(version: String) -> String {
    format!("https://maven.extframework.dev/releases/dev/extframework/client/{version}/client-{version}-all.jar", version = version)
}

pub async fn get_client(version: String) -> reqwest::Result<PathBuf> {
    let path = extframework_dir().join(format!("client-{}.jar", version));

    if !Path::new(path.as_os_str()).exists() {
        println!("Downloading client");
        download_client(version, &path).await?
    }
    Ok(path)
}

async fn download_client(version: String, path: &PathBuf) -> reqwest::Result<()> {
    let client_url = client_url(version);
    let response = reqwest::get(client_url).await?;

    create_dir_all(path.parent().unwrap()).expect("Failed to create client dir path");
    let mut client_file = File::create(path).expect("Failed to open client.jar file");
    let mut content = Cursor::new(response.bytes().await?);

    std::io::copy(&mut content, &mut client_file).expect("Failed to copy client.jar file");

    Ok(())
}

pub async fn get_client_version(path: &PathBuf) -> Result<String, ClientError> {
    let path = path.join("client_version.txt");

    let download_result: Result<(), ClientError> = {
        let request = reqwest::get("https://static.extframework.dev/client/latest_version")
            .await
            .map_err(NetworkError)?;

        let bytes = request.bytes().await.map_err(NetworkError)?;

        io::copy(
            &mut Cursor::new(bytes),
            &mut File::create(&path).map_err(IoError)?,
        )
        .map_err(IoError)?;
        Ok(())
    };

    if let Err(it) = download_result {
        if !&path.exists() {
            return Err(it);
        }
    }

    read_to_string(&path)
        .map(|it| it.trim().to_string())
        .map_err(IoError)
}

#[cfg(test)]
mod tests {
    use crate::launch::client::{get_client, get_client_version};
    use std::fs::create_dir_all;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_client_download() {
        get_client("1.0-SNAPSHOT".to_string()).await.unwrap();
    }

    #[tokio::test]
    async fn test_client_version() {
        let buf = PathBuf::from("client");
        create_dir_all(&buf).unwrap();
        println!("{}", get_client_version(&buf).await.unwrap());
    }
}
