#![feature(async_await)]

mod cargo;
mod command_ext;
mod git;
mod new;
mod output;
mod run;
mod serve;
mod sub_command;
mod watch;
mod watcher;

use crate::{new::New, run::Run, serve::Serve, sub_command::SubCommand, watch::Watch};
use std::{env, process};
use structopt::StructOpt;

pub type Result<T> = std::result::Result<T, failure::Error>;

/// Options for configuring fart.
#[derive(Clone, Debug, StructOpt)]
enum Options {
    /// Create a new fart project.
    #[structopt(name = "new")]
    New(New),

    /// Compile and run a fart project, to generate a new SVG.
    #[structopt(name = "run")]
    Run(Run),

    /// Watch a fart project for changes. On every change, rebuild the project,
    /// save an image, and make a commit.
    #[structopt(name = "watch")]
    Watch(Watch),

    /// Serve a fart project over a local HTTP server. Also watches, re-builds,
    /// and re-runs it on every change.
    #[structopt(name = "serve")]
    Serve(Serve),
}

impl SubCommand for Options {
    fn run(self) -> Result<()> {
        match self {
            Options::New(n) => n.run(),
            Options::Run(r) => r.run(),
            Options::Watch(w) => w.run(),
            Options::Serve(s) => s.run(),
        }
    }

    fn set_extra(&mut self, extra: &[String]) {
        match self {
            Options::New(n) => n.set_extra(extra),
            Options::Run(r) => r.set_extra(extra),
            Options::Watch(w) => w.set_extra(extra),
            Options::Serve(s) => s.set_extra(extra),
        }
    }
}

fn main() {
    // Since clap doesn't have good support for `-- blah ...` trailing
    // arguments, just split it off ourselves before giving it to structopt and
    // clap.
    let args: Vec<_> = env::args().collect();
    let args: Vec<_> = args.splitn(2, |a| a == "--").collect();
    let extra = args.get(1).cloned().unwrap_or(&[]);
    let args = args[0];

    let mut options = Options::from_iter(args);
    options.set_extra(extra);

    if let Err(e) = options.run() {
        eprintln!("Error: {}", e);
        for c in e.iter_causes() {
            eprintln!("    Caused by: {}", c);
        }
        process::exit(1);
    }
}
