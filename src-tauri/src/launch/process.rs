use std::{io, thread};
use std::env::args;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::from_utf8;
use std::sync::{Arc};

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::Manager;
use tokio::sync::Mutex;
use crate::launch::ClientError;
use crate::launch::ClientError::{IoError, JreInstallError};
use crate::launch::java::get_java_command;
use crate::minecraft_dir;
use crate::state::{Extension, MinecraftAuthentication};

#[derive(Clone, Serialize)]
pub struct ProcessStdoutEvent {
    pub is_err: bool,
    pub frag: Vec<u8>,
}

fn add_env_args(legacy: bool, command: &mut Command) -> &mut Command {
    #[cfg(target_os = "macos")] {
        if (!legacy) {
            command
                .arg("-XstartOnFirstThread");
        }
    }

    let bin_path = minecraft_dir().join("bin");
    command
        .arg(format!("-Djava.library.path={}", bin_path.to_str().unwrap()))
        // .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=*:5005")
        .arg("-jar")

}

struct ProcessStdEmitter {
    handle_ref: Channel<ProcessStdoutEvent>,
    is_err: bool,
}

impl<'a> Write for ProcessStdEmitter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        print!("{}", from_utf8(buf).unwrap());
        self.handle_ref.send(
            ProcessStdoutEvent {
                is_err: self.is_err,
                frag: Vec::from(buf),
            },
        ).unwrap();

        return Ok(buf.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub async fn launch_process(
    version: String,
    java_dir: PathBuf,
    client_path: PathBuf,
    auth: &Option<MinecraftAuthentication>,
    extensions: &Vec<Extension>,
) -> Result<Child, ClientError> {
    // TODO cleaner version support
    let legacy = version == "1.8.9";

    let java_version = if legacy {
        "8"
    } else { "21" };

    let os_name = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    let os_arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if (cfg!(target_arch = "aarch64") && !legacy) {
        "aarch64"
    } else {
        "x64"
    };

    let mut command = get_java_command(java_version, os_name, os_arch, java_dir).await.map_err(|it| {
        JreInstallError(it)
    })?;
    add_env_args(legacy, &mut command);
    command
        .arg(client_path.to_str().unwrap())
        .arg(format!("--version=extframework-{}", version))
        // Better stacktraces + TODO fixes access widening bug
        .arg("--mapping-namespace=mojang:deobfuscated");

    if let Some(auth) = auth {
        command
            .arg(format!("--accessToken={}", auth.access_token))
            .arg(format!("--uuid={}", auth.profile.id))
            .arg(format!("--username={}", auth.profile.name));
    }

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for x in extensions {
        command.arg("-e");
        command.arg(x.descriptor.as_str());
        command.arg("-r");
        command.arg(format!("{}@{}", x.repository_type.cli_arg(), x.repository.as_str()));
    }

    let child = command
        .spawn()
        .map_err(|e| IoError(e))?;

    Ok(child)
}

pub fn capture_child(
    mut child: Child,
    channel: Channel<ProcessStdoutEvent>,
) -> Arc<Mutex<Child>> {
    // Moved into the spawned thread.
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let child = Arc::new(Mutex::new(child));

    let clone1 = channel.clone();
    let mut emitter1 = ProcessStdEmitter {
        handle_ref: clone1,
        is_err: false,
    };

    thread::spawn(move || {
        loop {
            let mut buffer: [u8; 64] = [0; 64];
            // println!("Looping");
            if let Ok(length) = child_stdout.read(&mut buffer) {
                if length == 0 {
                    break;
                }

                emitter1.write(&buffer[0..length]).unwrap();
            }
        }
    });

    let clone2 = channel.clone();
    let mut emitter2 = ProcessStdEmitter {
        handle_ref: clone2,
        is_err: true,
    };

    thread::spawn(move || {
        loop {
            let mut buffer: [u8; 64] = [0; 64];
            // println!("Looping");
            if let Ok(length) = child_stderr.read(&mut buffer) {
                if length == 0 {
                    break;
                }

                emitter2.write(&buffer[0..length]).unwrap();
            }
        }
    });

    child
}