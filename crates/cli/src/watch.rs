use crate::{sub_command::SubCommand, watcher::Watcher, Result};
use std::path::PathBuf;
use std::process;
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
}

impl SubCommand for Watch {
    fn set_extra(&mut self, extra: &[String]) {
        assert!(self.extra.is_empty());
        self.extra = extra.iter().cloned().collect();
    }

    fn run(self) -> Result<()> {
        Watcher::new(self.project.clone())
            .extra(self.extra.clone())
            .on_rerun(move || {
                eprintln!("\n\n");
                for _ in 0..self.get_terminal_columns() {
                    eprint!("▔");
                }
                eprintln!("\n\n");
            })
            .watch()
    }
}
