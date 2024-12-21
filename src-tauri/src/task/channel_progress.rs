use crate::task::{Progress, TaskManager, TrackerBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::{command, AppHandle, Emitter, State};
use tokio::sync::Mutex;

pub struct ChannelProgressManager {
    pub channels: Mutex<HashMap<u64, Channel<ProgressData>>>,
}

#[derive(Serialize, Clone)]
pub struct TaskEvent {
    pub name: String,
    pub id: u64,
}

#[command]
pub async fn register_task_channel(
    id: u64,
    channel: Channel<ProgressData>,
    manager: State<'_, Arc<ChannelProgressManager>>,
) -> Result<(), ()> {
    manager.channels.lock().await.insert(id, channel);
    Ok(())
}

pub struct ChannelProgressBuilder {
    pub id: u64,
    pub manager: Arc<ChannelProgressManager>,
    pub handle: AppHandle,
}

impl TrackerBuilder for ChannelProgressBuilder {
    fn new(&mut self, name: &str) -> Progress {
        self.id = self.id + 1;

        self.manager.submit(
            &self.handle,
            name.to_string(),
            self.id,
        ).expect("Failed to submit progress tracker");

        Progress::Channel {
            percent: 0.0,
            erroneous: false,
            last_sent: 0.0,
            id: self.id,
            manager: Arc::clone(&self.manager),
        }
    }
}

impl ChannelProgressManager {
    pub fn new() -> Self {
        Self {
            channels: Default::default(),
        }
    }

    pub fn submit(&self, app: &AppHandle, name: String, id: u64) -> tauri::Result<()> {
        println!("Submitting progress for {}", name);
        app.emit("new-task", TaskEvent { name, id })
    }

    pub fn tasks(
        app: AppHandle,
    ) -> TaskManager {
        let manager = ChannelProgressManager {
            channels: Mutex::new(HashMap::new()),
        };

        let tasks = TaskManager {
            progress_builder: Box::new(ChannelProgressBuilder {
                id: 0,
                manager: Arc::new(manager),
                handle: app,
            })
        };

        tasks
    }
}

#[derive(Clone, Serialize)]
pub struct ProgressData {
    pub progress: f64,
    pub error: Option<String>,
}