use std::{env, io};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use zip::{CompressionMethod, ZipWriter};
use zip::write::SimpleFileOptions;

fn main() {
    tauri_build::build();

    if !Path::new("bin/jre/release").exists() {
        let mut jlink_command = Command::new("jlink");
        jlink_command
            .arg("--no-header-files")
            .arg("--no-man-pages")
            .arg("--compress=2")
            .arg("--strip-debug")
            .arg("--add-modules").arg("ALL-MODULE-PATH")
            .arg("--output").arg("./bin/jre")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let output = jlink_command.output().expect("Process didnt start correctly.");

        if !output.status.success() {
            panic!("JLink didnt complete successfully")
        }
    }

    let current_dir = env::current_dir().expect("Couldn't find working dir.");
    let path = current_dir
        .join("bin").join("jre.bundle");

    if !path.exists() {
        let file = File::create(path.clone()).expect("Failed to create file: bin/jre.zip");

        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755)
            .large_file(true);

        let mut writer = ZipWriter::new(file);
        let walked_path = current_dir.clone()
            .join("bin")
            .join("jre");
        let walkdir = walkdir::WalkDir::new(
            walked_path.clone()
        );

        for x in walkdir {
            let mut x = x.expect("Failed to walk dir");

            if x.file_type().is_dir() { continue; };

            let mut file1 = File::open(x.path()).expect("Failed to open file");

            writer.start_file(
                x.path()
                    .to_path_buf()
                    .strip_prefix(walked_path.clone()).unwrap().to_string_lossy(),
                options,
            ).expect("Failed to start file");

            io::copy(
                &mut file1,
                &mut writer,
            ).unwrap();
        }

        writer.finish().unwrap();
    }
}