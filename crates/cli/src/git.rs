use crate::{command_ext::CommandExt, output::Output, Result};
use failure::ResultExt;
use std::path::Path;
use std::process;

pub fn add_all<P>(dir: P, output: &mut Output) -> Result<()>
where
    P: AsRef<Path>,
{
    process::Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(dir)
        .run_result(output)
}

pub fn commit<P>(dir: P, msg: &str, output: &mut Output) -> Result<()>
where
    P: AsRef<Path>,
{
    process::Command::new("git")
        .arg("commit")
        .arg("--quiet")
        .arg("-m")
        .arg(msg)
        .current_dir(dir)
        .run_result(output)
}

pub fn any_staged<P>(dir: P) -> Result<bool>
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
