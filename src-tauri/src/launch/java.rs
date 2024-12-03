use crate::launch::java::JreSetupError::IOError;
use flate2::read::GzDecoder;
use std::fmt::{format, Debug, Display, Formatter};
use std::fs::{create_dir_all, File};
use std::io;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use zip_extract::{extract, ZipExtractError};

#[derive(Debug)]
pub enum JreSetupError {
    NetworkError(reqwest::Error),
    IOError(io::Error),
}

impl Display for JreSetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let message=  match self {
            JreSetupError::NetworkError(it) => { it.to_string() }
            IOError(it) => {it.to_string()}
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

    let java_command_path = jre_path.to_path_buf().join("Contents").join("Home").join("bin").join("java");

    if java_command_path.exists() {
        return Ok(java_command_path)
    }

    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/adoptium",
        version,
        os_name,
        os_arch
    );

    let bytes = reqwest::get(url).await
        .map_err(|it| JreSetupError::NetworkError(it))?.bytes().await.map_err(|it| JreSetupError::NetworkError(it))?;

    let tar = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(tar);

    archive
        .entries().map_err(IOError)?
        .filter_map(|e| e.ok())
        .map(|mut entry| -> io::Result<PathBuf> {
            // Strip first part of path
            let path = entry.path()?.strip_prefix(entry.path()?.components().next().unwrap()).unwrap().to_owned();
            entry.unpack(jre_path.join(&path))?;
            Ok(path)
        }).collect::<Result<Vec<_>, io::Error>>().map_err(IOError)?;

    Ok(java_command_path)
}

pub async fn get_java_command(
    version: &str,
    os_name: &str,
    os_arch: &str,
    path: PathBuf
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
    async fn test_download_jre_8() {
        let os_name = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        };


        let buf = PathBuf::from("jres");
        create_dir_all(&buf).await.unwrap();
        let path = download_jre("8", os_name, "x64", buf).await.unwrap();

        println!("{:?}", path);
    }
}