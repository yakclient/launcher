use std::fmt::{Debug, Display};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

mod channel_progress;
pub mod copy;
mod download;

pub struct TaskManager {
    progress_builder: Box<dyn TrackerBuilder>,
}

pub trait TrackerBuilder {
    fn new(&mut self, name: &str) -> Box<dyn ProgressUpdate>;
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

pub struct ChildProgressTracker {
    weight: f64,
    percent: f64,
    erroneous: bool,
    delegate: Arc<tokio::sync::Mutex<Box<dyn ProgressUpdate>>>,
}

impl ProgressTracker for ChildProgressTracker {
    fn percent(&self) -> f64 {
        self.percent
    }

    fn erroneous(&self) -> bool {
        self.erroneous
    }
}

impl ProgressUpdateAsync for ChildProgressTracker {
    async fn update(&mut self, progress: f64) {
        let mut guard = self.delegate.lock().await;

        let x = guard.percent();
        guard.update(x - (self.percent * self.weight) + (progress * self.weight));
        self.percent = progress;
    }

    async fn erroneously_complete(&mut self, err: &dyn Display) {
        self.delegate.lock().await.erroneously_complete(err)
    }
}

pub struct Task {
    pub name: String,
    pub progress: Box<dyn ProgressUpdate>,
}

impl Task {
    pub fn to_arc(self) -> Arc<tokio::sync::Mutex<Box<dyn ProgressUpdate>>> {
        Arc::new(tokio::sync::Mutex::new(self.progress))
    }

    pub fn child(
        tracker: &Arc<tokio::sync::Mutex<Box<dyn ProgressUpdate>>>,
        weight: f64,
    ) -> ChildProgressTracker {
        ChildProgressTracker {
            weight,
            percent: 0.0,
            erroneous: false,
            delegate: Arc::clone(tracker),
        }
    }

    fn update(&mut self, progress: f64) {
        self.progress.update(progress);
    }

    fn add(&mut self, progress: f64) {
        self.progress.add(progress);
    }
}

pub trait ProgressTracker {
    // A percent 0-1
    fn percent(&self) -> f64;

    fn completed(&self) -> bool {
        self.percent() == 1.0
    }

    fn erroneous(&self) -> bool;
}

pub trait ProgressUpdate: ProgressTracker {
    // A percent 0-1
    fn update(&mut self, progress: f64);

    fn add(&mut self, progress: f64) {
        self.update(self.percent() + progress);
    }

    fn erroneously_complete(&mut self, err: &dyn Display);
}

pub trait ProgressUpdateAsync: ProgressTracker {
    async fn update(&mut self, progress: f64);

    async fn add(&mut self, progress: f64) {
        self.update(self.percent() + progress).await;
    }

    async fn erroneously_complete(&mut self, err: &dyn Display);
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::task::{ProgressTracker, ProgressUpdate, TaskManager, TrackerBuilder};
    use std::fmt::Display;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::path::PathBuf;

    pub struct PrintingProgressTracker {
        pub percent: f64,
        pub erroneous: bool,
        pub file: File,
        pub name: String,
    }

    impl ProgressTracker for PrintingProgressTracker {
        fn percent(&self) -> f64 {
            self.percent
        }

        fn completed(&self) -> bool {
            self.percent == 1.0
        }

        fn erroneous(&self) -> bool {
            self.erroneous
        }
    }

    impl ProgressUpdate for PrintingProgressTracker {
        fn update(&mut self, progress: f64) {
            self.percent = progress;

            // self.file.write()
            use std::io::Write;
            writeln!(
                self.file,
                "{}: Progress: {:.1}%",
                self.name,
                progress * 100.0
            )
            .unwrap();
        }

        fn erroneously_complete(&mut self, err: &dyn Display) {
            self.erroneous = true;
            self.percent = 1.0;
            println!("Failed to complete.")
        }
    }

    pub struct PrintingTrackerBuilder {
        pub path: PathBuf,
    }

    impl TrackerBuilder for PrintingTrackerBuilder {
        fn new(&mut self, name: &str) -> Box<dyn ProgressUpdate> {
            create_dir_all(&self.path).unwrap();
            Box::new(PrintingProgressTracker {
                percent: 0f64,
                erroneous: false,
                file: File::create(self.path.join(format!("{}.txt", name))).unwrap(),
                name: name.to_string(),
            })
        }
    }

    #[test]
    fn test_child_progress_tracking() {
        let builder = PrintingTrackerBuilder {
            path: PathBuf::from("tests").join("child-test"),
        };

        let mut manager = TaskManager::new(Box::new(builder));

        // manager.submit("First task", |mut task: Task| {
        //     let mut child1 = task.child(0.25);
        //     child1.update(1.0);
        //
        //     task.add(0.5);
        //
        //     let mut child2 = task.child(0.25);
        //     child2.update(1.0);
        // });
    }
}
