use std::{io, thread};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::Manager;

use crate::launch::ClientError;
use crate::launch::ClientError::{IoError, ZipExtractError};
use crate::launch::java::get_java_command;
use crate::state::{Extension, MinecraftAuthentication};

#[derive(Clone, Serialize)]
pub struct ProcessStdoutEvent {
    pub is_err: bool,
    pub frag: Vec<u8>,
}

fn add_env_args(command: &mut Command) -> &mut Command {
    #[cfg(target_os = "macos")] {
        command
            .arg("-XstartOnFirstThread");
    }

    command
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

pub fn launch_process(
    version: String,
    java_dir: PathBuf,
    client_path: PathBuf,
    auth: &Option<MinecraftAuthentication>,
    extensions: &Vec<Extension>,
) -> Result<Child, ClientError> {
    let mut command = get_java_command(java_dir).map_err(|it| {
        ZipExtractError(it)
    })?;
    add_env_args(&mut command);
    command
        .arg(client_path.to_str().unwrap())
        .arg(format!("--version={}", version));

    if let Some(auth) = auth {
        command.arg(format!("--accessToken={}", auth.access_token))
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
        command.arg(format!("default@{}/registry", x.repository.as_str()));
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