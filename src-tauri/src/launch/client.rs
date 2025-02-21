use crate::extframework_dir;
use crate::launch::ClientError;
use crate::task::copy::copy_stream_tracking;
use crate::task::TaskManager;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

fn client_url(version: String) -> String {
    format!("https://maven.extframework.dev/releases/dev/extframework/client/{version}/client-{version}-all.jar", version = version)
}

pub async fn get_client(
    version: String,
    tasks: &mut TaskManager,
) -> Result<PathBuf, ClientError> {
    let path = extframework_dir().join(format!("client-{}.jar", version));

    if !Path::new(path.as_os_str()).exists() {
        println!("Downloading client");
        download_client(version, &path, tasks).await?
    }
    Ok(path)
}

async fn download_client(
    version: String,
    path: &PathBuf,
    tasks: &mut TaskManager,
) -> Result<(), ClientError> {

    tasks.submit("Download client", |mut task| {
        let client_url = client_url(version);
        async move {
            let response = reqwest::get(client_url).await
                .map_err(ClientError::NetworkError)?;

            let length = response.headers().get("Content-Length")
                .and_then(|t| Some(t.to_str().ok()?.parse::<u64>().ok()?))
                .unwrap_or(0u64);

            println!("{}", length);

            create_dir_all(path.parent().unwrap()).expect("Failed to create client dir path");
            let mut client_file = File::create(path).expect("Failed to open client.jar file");
            let mut stream = response.bytes_stream();

            let r : Result<(), ClientError> = copy_stream_tracking(
                &mut stream,
                &mut client_file,
                length,
                &mut task.progress
            ).await;
            r?;

            Ok::<(), ClientError>(())
        }
    }).await?;

    Ok(())
}

pub async fn get_client_version() -> Result<String, ClientError> {
    Ok("1.1.5-BETA".to_string())
}

#[cfg(test)]
mod tests {
    use crate::launch::client::{get_client, get_client_version};
    use std::fs::create_dir_all;
    use std::path::PathBuf;
    use crate::task::TaskManager;
    use crate::task::tests::PrintingTrackerBuilder;

    #[tokio::test]
    async fn test_client_download() {
        let builder = PrintingTrackerBuilder {
            path: PathBuf::from("tests").join("child-test"),
        };

        let mut manager = TaskManager::new(Box::new(builder));
        get_client("1.0.12-BETA".to_string(), &mut manager).await.unwrap();
    }

    #[tokio::test]
    async fn test_client_version() {
        let buf = PathBuf::from("client");
        create_dir_all(&buf).unwrap();
        println!("{}", get_client_version().await.unwrap());
    }
}
