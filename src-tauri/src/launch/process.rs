use std::{io, thread};
use std::io::{Read, stdout, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::launch::ClientError;
use crate::launch::ClientError::IoError;
use crate::state::MinecraftAuthentication;

#[derive(Clone, Serialize)]
pub struct ProcessStdoutEvent<'a> {
    pub is_err: bool,
    pub frag: &'a [u8],
}

fn add_env_args(command: &mut Command) -> &mut Command {
    command.arg("-XstartOnFirstThread")
        .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=32887")
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
    auth: &MinecraftAuthentication,
) -> Result<Child, ClientError> {
    let mut command = Command::new("java");
    add_env_args(&mut command);
    command
        .arg(client_path.to_str().unwrap())
        .arg(format!("--version={}", version))
        .arg(format!("--accessToken={}", auth.access_token))
        .arg(format!("--uuid={}", auth.profile.id))
        .arg(format!("--username={}", auth.profile.name))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

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

    let child_copy = child.clone();
    thread::spawn(move || {
        let mut std_console = ProcessStdEmitter {
            handle_ref: &app,
            is_err: false,
        };

        let mut std_err_console = ProcessStdEmitter {
            handle_ref: &app,
            is_err: true,
        };

        while child_copy.lock().unwrap().try_wait().unwrap().is_none() {
            fn transfer_to(
                channel_in: &mut impl Read,
                channel_out: &mut impl Write,
            ) -> io::Result<()> {
                let mut buffer: [u8; 64] = [0; 64];
                if let Ok(length) = channel_in.read(&mut buffer) {
                    channel_out.write(&buffer[0..length]).unwrap();
                }

                Ok(())
            }

            transfer_to(
                &mut child_stdout,
                &mut std_console,
            ).unwrap();

            // transfer_to(
            //     &mut child_stderr,
            //     &mut std_err_console,
            // ).unwrap();
        }
    });

    child
}