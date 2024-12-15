use crate::task::{ProgressTracker, ProgressUpdate};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Emitter, State};

pub struct ChannelProgressManager {
    channels: HashMap<u64, Channel<ProgressData>>,
}

#[derive(Serialize, Clone)]
pub struct TaskEvent {
    name: String,
    id: u64,
}

// #[command]
// fn register_task_channel(
//     id: u64,
//     channel: Channel<ProgressData>,
//     manager: State<'_, ChannelProgressManager>
// ) {
//     manager.channels.insert(id, channel);
// }

impl ChannelProgressManager {
    pub fn new() -> Self {
        Self {
            channels: Default::default(),
        }
    }

    pub fn submit(&self, app: &AppHandle, name: String, id: u64) -> tauri::Result<()> {
        app.emit("new_task", TaskEvent { name, id })
    }
}

pub struct ChannelProgress {
    channel: Option<Channel<ProgressData>>,
    progress: f64,
    erroneous: bool,
    id: u64,
    manager: Arc<ChannelProgressManager>,
}

impl ChannelProgress {
    fn check_handle(&mut self) {
        if self.channel.is_none() {
            // if let Some(channel) =  self.manager.channels.get(&self.id) {
            //     let x: &Channel<ProgressData> = channel;
            //     let c: Channel<ProgressData>= x.clone();
            //     self.channel = Some();
            // }
        }
    }

    // pub fn send_new(
    //     name: String,
    //     channel: Channel<ProgressData>,
    //     app: &AppHandle,
    // ) -> DisplayableProgress {
    //     app.emit("tasks", TaskEvent {
    //         name,
    //         channel,
    //     });
    //
    //     DisplayableProgress {
    //         channel,
    //         progress: 0.0,
    //         erroneous: false,
    //     }
    // }
}

#[derive(Serialize)]
struct ProgressData {
    progress: f64,
    error: Option<String>,
}

impl ProgressTracker for ChannelProgress {
    fn percent(&self) -> f64 {
        self.progress
    }

    fn completed(&self) -> bool {
        self.progress == 1.0
    }

    fn erroneous(&self) -> bool {
        self.erroneous
    }
}

impl ProgressUpdate for ChannelProgress {
    fn update(&mut self, progress: f64) {
        self.check_handle();
        self.progress = progress;

        if let Some(channel) = &self.channel {
            channel
                .send(ProgressData {
                    progress,
                    error: None,
                })
                .expect("Failed to send progress update");
        }
    }

    fn erroneously_complete(&mut self, err: &dyn Display) {
        self.check_handle();
        self.progress = 1.0;
        self.erroneous = true;

        if let Some(channel) = &self.channel {
            channel
                .send(ProgressData {
                    progress: 1.0,
                    error: Some(err.to_string()),
                })
                .expect("Failed to send progress update");
        }
    }
}
