use failure::{bail, ResultExt};
use fart::Result;
use notify::Watcher;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc;
use std::time;
use structopt::StructOpt;

trait Command {
    fn run(&mut self) -> Result<()>;

    fn set_extra(&mut self, extra: &[String]) {
        let _ = extra;
    }
}

/// Options for configuring fart.
#[derive(Clone, Debug, StructOpt)]
enum Options {
    /// Create a new fart project.
    #[structopt(name = "new")]
    New(New),

    /// Watch a fart project for changes. On every change, rebuild the project,
    /// save an image, and make a commit.
    #[structopt(name = "watch")]
    Watch(Watch),
}

impl Command for Options {
    fn run(&mut self) -> Result<()> {
        match self {
            Options::New(n) => n.run(),
            Options::Watch(w) => w.run(),
        }
    }

    fn set_extra(&mut self, extra: &[String]) {
        match self {
            Options::New(n) => n.set_extra(extra),
            Options::Watch(w) => w.set_extra(extra),
        }
    }
}

/// Create a new fart project.
#[derive(Clone, Debug, StructOpt)]
struct New {
    /// The name of the new project.
    #[structopt(parse(from_os_str))]
    name: PathBuf,

    /// The fart project template to use.
    #[structopt(default_value = "https://github.com/fitzgen/fart-template.git")]
    template: String,
}

impl Command for New {
    fn run(&mut self) -> Result<()> {
        process::Command::new("git")
            .arg("clone")
            .arg(&self.template)
            .arg(&self.name)
            .run_result()?;
        eprintln!(
            "\nCreated new fart project: {}",
            self.name.canonicalize()?.display()
        );
        Ok(())
    }
}

/// Watch a fart project for changes.
#[derive(Clone, Debug, StructOpt)]
struct Watch {
    /// The project to watch.
    #[structopt(parse(from_os_str), default_value = ".")]
    project: PathBuf,

    /// Extra arguments passed along to each invocation of `cargo run`.
    #[structopt(long = "")]
    extra: Vec<String>,
}

impl Watch {
    fn on_file_change(&self) -> Result<()> {
        eprintln!(
            "\n\n--------------------------------------------------------------------------------"
        );
        let now = chrono::Utc::now();
        let now = now.format("%Y-%m-%d-%H-%M-%S-%f").to_string();

        let images = self.project.join("images");
        fs::create_dir_all(&images)
            .with_context(|_| format!("failed to create directory: {}", images.display()))?;

        let mut file_name = images.join(&now);
        file_name.set_extension("svg");
        let file_name = file_name.canonicalize().unwrap_or(file_name);

        cargo_build(&self.project, &self.extra)?;

        git_add_all(&self.project)?;
        if !any_staged_in_git(&self.project)? {
            return Ok(());
        }

        cargo_run(
            &self.project,
            &self.extra,
            vec![("FART_FILE_NAME", &file_name)],
        )?;

        link_as_latest(&self.project, &file_name)?;

        git_add_all(&self.project)?;
        git_commit(&self.project, &now)?;
        Ok(())
    }
}

impl Command for Watch {
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

trait CommandExt {
    fn run_result(self) -> Result<()>;
}

impl CommandExt for &'_ mut process::Command {
    fn run_result(self) -> Result<()> {
        let status = self
            .status()
            .with_context(|_| format!("failed to execute: {:?}", self))?;
        if !status.success() {
            bail!("command exited unsuccessfully: {:?}", self);
        }
        Ok(())
    }
}

fn cargo_build<P, I, A>(dir: P, args: I) -> Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = A>,
    A: AsRef<OsStr>,
{
    process::Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(dir.as_ref().join("Cargo.toml"))
        .args(args)
        .run_result()
}

fn cargo_run<P, I, A, E, K, V>(dir: P, args: I, envs: E) -> Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = A>,
    A: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    process::Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(dir.as_ref().join("Cargo.toml"))
        .args(args)
        .envs(envs)
        .run_result()
}

fn link_as_latest<P, Q>(project: P, img: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let img = img.as_ref();
    let latest = project.as_ref().join("images").join("latest.svg");
    let _ = fs::remove_file(&latest);
    fs::hard_link(img, &latest)
        .with_context(|_| format!("failed to link {} to {}", img.display(), latest.display()))?;
    eprintln!("\nLinked {} to {}", img.display(), latest.display());
    Ok(())
}

fn git_add_all<P>(dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    process::Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(dir)
        .run_result()
}

fn git_commit<P>(dir: P, msg: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    process::Command::new("git")
        .arg("commit")
        .arg("--quiet")
        .arg("-m")
        .arg(msg)
        .current_dir(dir)
        .run_result()
}

fn any_staged_in_git<P>(dir: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    let status = process::Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--quiet")
        .current_dir(dir)
        .status()
        .context("failed to spawn `git diff`")?;
    Ok(!status.success())
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
