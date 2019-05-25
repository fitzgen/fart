use crate::{command_ext::CommandExt, output::Output, Result};
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
