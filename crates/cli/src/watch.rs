use crate::{git, run::Run, sub_command::SubCommand, Result};
use failure::ResultExt;
use notify::Watcher;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;
use std::time;
use structopt::StructOpt;

/// Watch a fart project for changes.
#[derive(Clone, Debug, StructOpt)]
pub struct Watch {
    /// The project to watch.
    #[structopt(parse(from_os_str), default_value = ".")]
    project: PathBuf,

    /// Extra arguments passed along to each invocation of `cargo run`.
    #[structopt(long = "")]
    extra: Vec<String>,
}

impl Watch {
    pub fn new(project: PathBuf, extra: Vec<String>) -> Watch {
        Watch { project, extra }
    }

    fn get_terminal_columns(&self) -> usize {
        let tput = || -> Result<usize> {
            let out = process::Command::new("tput").arg("cols").output()?;
            failure::ensure!(out.status.success(), "`tput` did not exit successfully");
            let s = String::from_utf8(out.stdout)?;
            let n = str::parse::<usize>(s.trim())?;
            Ok(n - 1)
        };

        tput().unwrap_or(80)
    }

    fn on_file_change(&self) -> Result<()> {
        git::add_all(&self.project)?;
        if !git::any_staged(&self.project)? {
            return Ok(());
        }

        eprintln!("\n\n");
        for _ in 0..self.get_terminal_columns() {
            eprint!("▔");
        }
        eprintln!("\n\n");

        Run::new(self.project.clone(), self.extra.clone()).run()
    }
}

impl SubCommand for Watch {
    fn set_extra(&mut self, extra: &[String]) {
        assert!(self.extra.is_empty());
        self.extra = extra.iter().cloned().collect();
    }

    fn run(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::watcher(tx, time::Duration::from_millis(100))
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
        eprintln!("Watching fart project for changes: {}", project.display());

        loop {
            // Wait for a file to be updated or whatever.
            let _ = rx
                .recv()
                .context("failed to receive file watcher message")?;

            // Drain the channel so we don't build again until we get
            // notifications from after we build.
            while let Ok(_) = rx.try_recv() {}

            if let Err(e) = self.on_file_change() {
                eprintln!("Warning: {}", e);
                for c in e.iter_causes() {
                    eprintln!("    Caused by: {}", c);
                }
            }
        }
    }
}
