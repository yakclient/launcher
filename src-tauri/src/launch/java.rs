use crate::launch::java::JreSetupError::{IOError, NetworkError, UnsuccessfulZuluQuery, ZipError};
use futures::StreamExt;
use serde::Deserialize;
use std::fmt::{Debug, Display, Formatter};
use std::fs::{create_dir_all, File};
use std::{fs, io};
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;

#[derive(Debug)]
pub enum JreSetupError {
    NetworkError(reqwest::Error),
    IOError(io::Error),
    ZipError(zip::result::ZipError),
    UnsuccessfulZuluQuery,
}

impl Display for JreSetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            JreSetupError::NetworkError(it) => it.to_string(),
            IOError(it) => it.to_string(),
            ZipError(it) => it.to_string(),
            UnsuccessfulZuluQuery => "Failed to query Zulu for an appropriate JDK to download!".to_string()
        };

        write!(f, "Failed to download JDK because of {}", message)
    }
}

async fn download_jre(
    version: &str,
    os_name: &str,
    os_arch: &str,
    path: PathBuf,
) -> Result<PathBuf, JreSetupError> {
    let jre_path = path.join(format!("jre-{}", version));

    let java_command_path = if os_name == "windows" {
        jre_path.to_path_buf().join("bin").join("java.exe")
    } else {
        jre_path
            .to_path_buf()
            .join(format!("zulu-{}.jre", version))
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    };

    if java_command_path.exists() {
        return Ok(java_command_path);
    }

    let client = reqwest::Client::new();

    let url = get_download_url(
        &client,
        version, os_name, os_arch,
    ).await?;

    let bytes = client.get(url)
        .send()
        .await
        .map_err(|it| NetworkError(it))?
        .bytes()
        .await
        .map_err(|it| NetworkError(it))?;

    let cursor = Cursor::new(bytes);

    // if os_name == "windows" {
        extract_zip(jre_path, cursor)?;
    // } else {
    //     extract_tar_gz(jre_path, cursor)?;
    // }

    #[cfg(target_os = "macos")] {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let metadata = fs::metadata(&java_command_path).unwrap();
        let mut permissions = metadata.permissions();

        permissions.set_mode(0o770);

        fs::set_permissions(&java_command_path, permissions).unwrap();
    }

    Ok(java_command_path)
}

#[derive(Deserialize)]
struct ZuluJreResponse {
    download_url: String,
}

async fn get_download_url(
    client: &reqwest::Client,
    version: &str,
    os_name: &str,
    os_arch: &str,
) -> Result<String, JreSetupError> {

    let download_type = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar_gz"
    };

    let url = format!(
        "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type={}&javafx_bundled=false&java_package_type=jre&page_size=1",
        os_arch, version, os_name, "zip"
    );

    println!("Downloading {}", url);

    let response = client.get(&url).send().await.map_err(NetworkError)?;

    if response.status().is_success() {
        let res: Vec<ZuluJreResponse> = response.json().await.map_err(NetworkError)?;

        Ok(res.get(0).ok_or(UnsuccessfulZuluQuery)?.download_url.clone())
    } else {
        Err(UnsuccessfulZuluQuery)
    }
}

fn extract_zip(jre_path: PathBuf, cursor: Cursor<bytes::Bytes>) -> Result<(), JreSetupError> {
    let mut zip = ZipArchive::new(cursor).map_err(ZipError)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(ZipError)?;
        let original_path = file.sanitized_name();

        // Strip the first part of the path
        let stripped_path = original_path
            .components()
            .skip(1) // Skip the top-level component
            .collect::<PathBuf>();

        let outpath = jre_path.join(stripped_path);

        if file.is_dir() {
            create_dir_all(&outpath).map_err(IOError)?;
        } else {
            if let Some(parent) = outpath.parent() {
                create_dir_all(parent).map_err(IOError)?;
            }
            let mut outfile = File::create(&outpath).map_err(IOError)?;
            std::io::copy(&mut file, &mut outfile).map_err(IOError)?;
        }
    }
    Ok(())
}
fn extract_tar_gz(jre_path: PathBuf, cursor: Cursor<bytes::Bytes>) -> Result<(), JreSetupError> {
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    archive
        .entries()
        .map_err(IOError)?
        .map(|mut entry| -> io::Result<PathBuf> {
            let mut entry = entry?;
            println!("{}", entry.path().unwrap().display());
            // Strip first part of path
            let path = entry
                .path()?
                .strip_prefix(entry.path()?.components().next().unwrap())
                .unwrap()
                .to_owned();
            entry.unpack(jre_path.join(&path))?;
            Ok(path)
        })
        .collect::<Result<Vec<_>, io::Error>>()
        .map_err(IOError)?;
    Ok(())
}

pub async fn get_java_command(
    version: &str,
    os_name: &str,
    os_arch: &str,
    path: PathBuf,
) -> Result<Command, JreSetupError> {
    let path = download_jre(version, os_name, os_arch, path).await?;

    Ok(Command::new(path))
}

#[cfg(test)]
mod tests {
    use crate::launch::java::download_jre;
    use std::path::PathBuf;
    use tokio::fs::create_dir_all;

    #[tokio::test]
    async fn test_download_jdk() {
        let os_name = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        };
        let os_arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x64"
        };
        let buf = PathBuf::from("jres");
        create_dir_all(&buf).await.unwrap();
        let path = download_jre("21", os_name, os_arch, buf).await.unwrap();

        println!("{:?}", path);
    }

    #[tokio::test]
    async fn test_windows_jre() {
        let buf = PathBuf::from("jres");
        create_dir_all(&buf).await.unwrap();
        let path = download_jre("21", "windows", "x64", buf).await.unwrap();

        println!("{:?}", path);
    }

    #[tokio::test]
    async fn test_download_jre_8() {
        let os_name = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        };

        let os_arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x64"
        };

        let buf = PathBuf::from("jres");
        create_dir_all(&buf).await.unwrap();
        let path = download_jre("8", os_name, os_arch, buf).await.unwrap();

        println!("{:?}", path);
    }
}
