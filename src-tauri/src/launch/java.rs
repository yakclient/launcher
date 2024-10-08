use std::fs::create_dir_all;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use zip_extract::{extract, ZipExtractError};

fn write_jdk(path: &PathBuf) -> Result<(), ZipExtractError> {
    let jre = include_bytes!("../../bin/jre.bundle");

    extract(Cursor::new(jre), Path::new(path), false).unwrap();
    Ok(())
}

pub fn get_java_command(
    path: PathBuf
) -> Result<Command, ZipExtractError> {
    let path_to_java = path
        .join("jre")
        .join("bin")
        .join("java");

    if !Path::new(&path_to_java).exists() {
        let buf = path.join("jre");
        create_dir_all(&buf).map_err(|e| {
            ZipExtractError::Io(e)
        })?;
        write_jdk(
            &buf
        )?;
    }

    Ok(Command::new(path_to_java))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::launch::java::get_java_command;

    #[test]
    fn test_get_java_command() {
        get_java_command(
            PathBuf::from("test")
        ).unwrap();
    }
}