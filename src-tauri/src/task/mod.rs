use crate::task::channel_progress::{ChannelProgressManager, ProgressData};
use std::fmt::{Debug, Display};
use std::fs::File;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub mod channel_progress;
pub mod copy;
mod download;

pub struct TaskManager {
    pub progress_builder: Box<dyn TrackerBuilder>,
}

pub trait TrackerBuilder: Send {
    fn new(&mut self, name: &str) -> Progress;
}

impl TaskManager {
    pub fn new(builder: Box<dyn TrackerBuilder>) -> TaskManager {
        TaskManager {
            progress_builder: builder,
        }
    }

    pub fn submit<F, T, N>(&mut self, name: N, block: F) -> T
    where
        F: FnOnce(Task) -> T,
        N: Into<String>,
    {
        let name = name.into();
        let tracker = self.progress_builder.new(name.as_str());

        let task = Task {
            name,
            progress: tracker,
        };

        let ret = block(task);

        ret
    }
}

pub enum Progress {
    Channel {
        percent: f64,
        erroneous: bool,
        last_sent: f64,
        id: u64,
        manager: Arc<ChannelProgressManager>,
    },
    Logging {
        percent: f64,
        erroneous: bool,
        file: File,
        name: String,
    },
    Child {
        percent: f64,
        erroneous: bool,
        weight: f64,
        delegate: Arc<tokio::sync::Mutex<Progress>>,
    },
}

impl Progress {
    pub fn percent(&self) -> f64 {
        *match self {
            Progress::Channel { percent, .. } => { percent }
            Progress::Logging { percent, .. } => { percent }
            Progress::Child { percent, .. } => { percent }
        }
    }

    pub fn completed(&self) -> bool {
        self.percent() == 1.0
    }

    pub fn erroneous(&self) -> bool {
        *match self {
            Progress::Channel { erroneous, .. } => { erroneous }
            Progress::Logging { erroneous, .. } => { erroneous }
            Progress::Child { erroneous, .. } => { erroneous }
        }
    }

    pub async fn update(&mut self, progress: f64) {
        match self {
            Progress::Channel { percent, manager, id, last_sent, .. } => {
                *percent = progress;

                let guard = manager.channels.lock().await;
                let channel = guard.get(id);

                if progress - *last_sent >= 0.01f64 || progress >= 1f64 {
                    *last_sent = progress;
                    if let Some(channel) = channel {
                        channel
                            .send(ProgressData {
                                progress,
                                error: None,
                            }).expect("Failed to send progress update");
                    }
                }
            }
            Progress::Logging { percent, file, name, .. } => {
                *percent = progress;

                use std::io::Write;
                writeln!(
                    file,
                    "{}: Progress: {:.1}%",
                    name,
                    progress * 100.0
                ).unwrap();
            }
            Progress::Child { delegate, percent, weight, .. } => {
                let mut guard = delegate.lock().await;

                let x = guard.percent();
                Box::pin(guard.update(x - (*percent * *weight) + (progress * *weight))).await;
                *percent = progress;
            }
        }
    }

    pub async fn add(&mut self, progress: f64) {
        self.update(self.percent() + progress).await;
    }

    pub async fn erroneously_complete(&mut self, err: &(dyn Display + Send + Sync)) {
        match self {
            Progress::Channel { percent, erroneous, manager, id, .. } => {
                *percent = 1.0;
                *erroneous = true;

                let guard = manager.channels.lock().await;
                let channel = guard.get(id);

                if let Some(channel) = channel {
                    channel
                        .send(ProgressData {
                            progress: 1.0,
                            error: Some(err.to_string()),
                        }).expect("Failed to send progress update");
                }
            }
            Progress::Logging { erroneous, percent, .. } => {
                *erroneous = true;
                *percent = 1.0;
            }
            Progress::Child { delegate, .. } => {
                let mut guard = delegate.lock().await;
                Box::pin(guard.erroneously_complete(err)).await;
            }
        }
    }
}

pub struct Task {
    pub name: String,
    pub progress: Progress,
}

impl Task {
    pub fn to_arc(self) -> Arc<tokio::sync::Mutex<Progress>> {
        Arc::new(tokio::sync::Mutex::new(self.progress))
    }

    pub fn child(
        tracker: &Arc<tokio::sync::Mutex<Progress>>,
        weight: f64,
    ) -> Progress {
        Progress::Child {
            weight,
            percent: 0.0,
            erroneous: false,
            delegate: Arc::clone(tracker),
        }
    }

    async fn update(&mut self, progress: f64) {
        self.progress.update(progress).await;
    }

   async fn add(&mut self, progress: f64) {
        self.progress.add(progress).await;
    }
}
//
// pub trait ProgressTracker {
//     // A percent 0-1
//     fn percent(&self) -> f64;
//
//     fn completed(&self) -> bool {
//         self.percent() == 1.0
//     }
//
//     fn erroneous(&self) -> bool;
// }
//
// pub trait ProgressUpdate: ProgressTracker + Send {
//     // A percent 0-1
//     fn update(&mut self, progress: f64);
//
//     fn add(&mut self, progress: f64) {
//         self.update(self.percent() + progress);
//     }
//
//     fn erroneously_complete(&mut self, err: &dyn Display + Send);
// }
//
// pub trait ProgressUpdateAsync: ProgressTracker {
//     async fn update(&mut self, progress: f64);
//
//     async fn add(&mut self, progress: f64) {
//         self.update(self.percent() + progress).await;
//     }
//
//     async fn erroneously_complete(&mut self, err: &dyn Display + Send);
// }

#[cfg(test)]
pub(crate) mod tests {
    use crate::task::{Progress, Task, TaskManager, TrackerBuilder};
    use std::fmt::Display;
    use std::fs::{create_dir_all, File};
    use std::path::PathBuf;


    pub struct PrintingTrackerBuilder {
        pub path: PathBuf,
    }

    impl TrackerBuilder for PrintingTrackerBuilder {
        fn new(&mut self, name: &str) -> Progress {
            create_dir_all(&self.path).unwrap();
            Progress::Logging {
                percent: 0f64,
                erroneous: false,
                file: File::create(self.path.join(format!("{}.txt", name))).unwrap(),
                name: name.to_string(),
            }
        }
    }

    #[test]
    fn test_child_progress_tracking() {
        let builder = PrintingTrackerBuilder {
            path: PathBuf::from("tests").join("child-test"),
        };

        let mut manager = TaskManager::new(Box::new(builder));

        manager.submit("First task", |mut task: Task| {
            // let mut child1 = task.child(0.25);
            // child1.update(1.0);
            //
            // task.add(0.5);
            //
            // let mut child2 = task.child(0.25);
            // child2.update(1.0);
        });
    }
}
