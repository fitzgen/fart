use crate::{command_ext::CommandExt, output::Output, Result};
use std::ffi::OsStr;
use std::path::Path;
use std::process;

pub fn build<P, I, A>(dir: P, args: I, output: &mut Output) -> Result<()>
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
        .run_result(output)
}

pub fn run<P, I, A, E, K, V>(dir: P, args: I, envs: E, output: &mut Output) -> Result<()>
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
        .arg("--release")
        .arg("--manifest-path")
        .arg(dir.as_ref().join("Cargo.toml"))
        .args(args)
        .envs(envs)
        .run_result(output)
}
