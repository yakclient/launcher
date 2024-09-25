use std::{io, thread};
use std::io::{Read, stderr, stdout, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::launch::ClientError;
use crate::launch::ClientError::IoError;
use crate::state::{Extension, MinecraftAuthentication};

#[derive(Clone, Serialize)]
pub struct ProcessStdoutEvent<'a> {
    pub is_err: bool,
    pub frag: &'a [u8],
}

fn add_env_args(command: &mut Command) -> &mut Command {
    command.arg("-XstartOnFirstThread")
        // .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=32887")
        .arg("-jar")
}

struct ProcessStdEmitter<'a> {
    handle_ref: &'a AppHandle,
    is_err: bool,
}

impl<'a> Write for ProcessStdEmitter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        print!("{}", from_utf8(buf).unwrap());

        self.handle_ref.emit_all(
            "process-stdout",
            ProcessStdoutEvent {
                is_err: self.is_err,
                frag: buf,
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
    client_path: PathBuf,
    auth: &Option<MinecraftAuthentication>,
    extensions: &Vec<Extension>,
) -> Result<Child, ClientError> {
    let mut command = Command::new("java");
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
    app: AppHandle,
) -> Arc<Mutex<Child>> {
    // Moved into the spawned thread.
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let child = Arc::new(Mutex::new(child));

    let clone = app.clone();
    thread::spawn(move || {
        let mut emitter = ProcessStdEmitter {
            handle_ref: &clone,
            is_err: false,
        };

        loop {
            let mut buffer: [u8; 64] = [0; 64];
            // println!("Looping");
            if let Ok(length) = child_stdout.read(&mut buffer) {
                if length == 0 {
                    break;
                }

                emitter.write(&buffer[0..length]).unwrap();
            }
        }
    });

    let clone = app.clone();
    thread::spawn(move || {
        let mut emitter = ProcessStdEmitter {
            handle_ref: &clone,
            is_err: true,
        };

        loop {
            let mut buffer: [u8; 64] = [0; 64];
            if let Ok(length) = child_stderr.read(&mut buffer) {
                if length == 0 {
                    break;
                }
                emitter.write(&buffer[0..length]).unwrap();
            }
        }
    });

    child
}