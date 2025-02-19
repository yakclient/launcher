use crate::launch::java::get_java_command;
use crate::launch::minecraft::{Argument, Arguments, FormatForCommand, MinecraftEnvironment, ValueType};
use crate::launch::ClientError;
use crate::launch::ClientError::{IoError, JreInstallError};
use crate::minecraft_dir;
use crate::state::{Extension, MinecraftAuthentication};
use serde::Serialize;
use std::collections::HashMap;
use std::env::args;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::from_utf8;
use std::sync::Arc;
use std::{io, thread};
use std::thread::sleep;
use std::time::Duration;
use tauri::ipc::Channel;
use tauri::Manager;
use tokio::sync::Mutex;
use crate::settings::DebuggerSettings;

#[derive(Clone, Serialize)]
pub struct ProcessStdoutEvent {
    pub is_err: bool,
    pub frag: Vec<u8>,
}

// fn add_env_args<'a, 'b>(
//     legacy: bool,
//     command: &'b mut Command,
//     env: &'a MinecraftEnvironment
// ) -> &'b mut Command {
//     #[cfg(target_os = "macos")]
//     {
//         if !legacy {
//             command.arg("-XstartOnFirstThread");
//         }
//     }
//
//     let bin_path = minecraft_dir().join("bin");
//     command
//         .arg(format!(
//             "-Djava.library.path={}",
//             bin_path.to_str().unwrap()
//         ))
//         // .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address=*:5005")
//         .arg("-jar")
// }

struct ProcessStdEmitter {
    handle_ref: Channel<ProcessStdoutEvent>,
    is_err: bool,
}

impl<'a> Write for ProcessStdEmitter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        print!("{}", from_utf8(buf).unwrap());
        self.handle_ref
            .send(ProcessStdoutEvent {
                is_err: self.is_err,
                frag: Vec::from(buf),
            })
            .unwrap();

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
    env: &MinecraftEnvironment,
    debugger_settings: DebuggerSettings,
) -> Result<Child, ClientError> {
    // TODO cleaner version support
    let java_version = env.java_version.major_version.to_string();

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

    let mut classpath = env.libraries.clone();
    classpath.push(env.client_jar.clone());
    // classpath.push(client_path);

    let mut arg_variables = HashMap::from([
        ("version", version.clone()),
        ("version_name", version.clone()),
        ("game_directory", minecraft_dir().to_str().unwrap().to_string()),
        ("assets_root", env.asset_path.to_str().unwrap().to_string()),
        ("assets_index_name", env.asset_index_name.clone()),
        ("natives_directory", env.natives_path.to_str().unwrap().to_string()),
        ("launcher_name", "yakclient".to_string()),
        ("classpath", "~/nothing.jar".to_string()) // Just any temporary placeholder
    ]);

    if let Some(auth) = auth {
        arg_variables.insert("auth_player_name", auth.profile.name.clone());
        arg_variables.insert("auth_uuid", auth.profile.id.clone());
        arg_variables.insert("auth_access_token", auth.access_token.clone());
    }

    let mut command = get_java_command(java_version.as_str(), os_name, os_arch, java_dir)
        .await
        .map_err(|it| JreInstallError(it))?;

    command.current_dir(minecraft_dir());

    env.arguments.jvm
        .apply(
            &mut command,
            &arg_variables,
        );

    if debugger_settings.enabled {
        command.arg(
            format!(
                "-agentlib:jdwp=transport=dt_socket,server=y,suspend={},address={}",
                if debugger_settings.suspend { "y" } else { "n" },
                debugger_settings.port
            )
        );
    }

    command
        .arg("-jar")
        .arg(client_path.to_str().unwrap())
        .arg("--main-class")
        .arg(&env.main_class)
        .arg("--mapping-namespace")
        .arg("mojang:obfuscated")
        .arg("--classpath")
        .arg(classpath.iter().map(|s| s.to_str().unwrap().to_string()).collect::<Vec<String>>().join(";"))
        .arg("--game-jar")
        .arg(env.client_jar.to_str().unwrap())
        .arg("--version")
        .arg(version);

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    for x in extensions {
        command.arg("-e");
        command.arg(x.descriptor.as_str());
        command.arg("-r");
        command.arg(format!(
            "{}@{}",
            x.repository_type.cli_arg(),
            x.repository.as_str()
        ));
    }

    // Separator from MC args
    command.arg(":");

    env.arguments.game
        .chunks(2)
        .collect::<Vec<&[Argument]>>()
        .apply(
            &mut command,
            &arg_variables,
        );

    let child = command.spawn().map_err(|e| IoError(e))?;

    Ok(child)
}

pub fn capture_child(mut child: Child, channel: Channel<ProcessStdoutEvent>) -> Arc<Mutex<Child>> {
    // Moved into the spawned thread.
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let child = Arc::new(Mutex::new(child));

    let clone1 = channel.clone();
    let mut emitter1 = ProcessStdEmitter {
        handle_ref: clone1,
        is_err: false,
    };

    // TODO not super efficient
    thread::spawn(move || {
        loop {
            sleep(Duration::from_millis(2));
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
            sleep(Duration::from_millis(2));
            let mut buffer: [u8; 64] = [0; 64];
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

