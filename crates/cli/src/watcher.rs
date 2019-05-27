//! Watching, re-building, and re-running `fart` projects.

use crate::{output::Output, run::Run, Result};
use failure::ResultExt;
use notify::Watcher as _;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time;

pub struct Watcher {
    project: PathBuf,
    extra: Vec<String>,
    output: Output,
    on_start: Option<Box<FnMut()>>,
    on_finish: Option<Box<FnMut()>>,
}

impl Watcher {
    pub fn new<P>(project: P) -> Watcher
    where
        P: Into<PathBuf>,
    {
        let project = project.into();
        Watcher {
            project,
            extra: Default::default(),
            output: Output::Inherit,
            on_start: None,
            on_finish: None,
        }
    }

    pub fn extra(&mut self, extra: Vec<String>) -> &mut Self {
        self.extra = extra;
        self
    }

    pub fn on_output(&mut self, f: impl 'static + Send + FnMut(&str)) -> &mut Self {
        self.output = Output::Pipe(Arc::new(Mutex::new(f)));
        self
    }

    pub fn on_start(&mut self, f: impl 'static + FnMut()) -> &mut Self {
        self.on_start = Some(Box::new(f) as Box<FnMut()>);
        self
    }

    pub fn on_finish(&mut self, f: impl 'static + FnMut()) -> &mut Self {
        self.on_finish = Some(Box::new(f) as Box<FnMut()>);
        self
    }

    pub fn watch(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::watcher(tx, time::Duration::from_millis(50))
            .context("failed to create file watcher")?;

        watcher
            .watch(self.project.join("src"), notify::RecursiveMode::Recursive)
            .with_context(|_| {
                format!(
                    "failed to recursively add directory for watching: {}",
                    self.project.display()
                )
            })?;

        let project = self
            .project
            .canonicalize()
            .unwrap_or_else(|_| self.project.clone());
        writeln!(
            &mut self.output,
            "Watching fart project for changes: {}",
            project.display()
        )?;

        loop {
            // Wait for a file to be updated or whatever.
            let _ = rx
                .recv()
                .context("failed to receive file watcher message")?;

            // Drain the channel so we don't build again until we get
            // notifications from after we build.
            while let Ok(_) = rx.try_recv() {}

            if let Err(e) = self.rerun() {
                write!(&mut self.output, "Warning: {}", e)?;
                for c in e.iter_causes() {
                    write!(&mut self.output, "    Caused by: {}", c)?;
                }
            }
        }
    }

    fn rerun(&mut self) -> Result<()> {
        if let Some(f) = self.on_start.as_mut() {
            f();
        }

        let result =
            Run::new(self.project.clone(), self.extra.clone()).run_with_output(&mut self.output);

        if let Some(f) = self.on_finish.as_mut() {
            f();
        }

        result
    }
}
