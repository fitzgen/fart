use crate::{output::Output, Result};
use failure::{bail, ResultExt};
use std::{
    io::{self, BufRead, Write},
    process, thread,
};

/// Extension trait for `std::process::Command`.
pub trait CommandExt {
    /// Run the command and get a result based on if it completed successfully
    /// or not.
    fn run_result(self, output: &mut Output) -> Result<()>;
}

impl CommandExt for &'_ mut process::Command {
    fn run_result(self, output: &mut Output) -> Result<()> {
        if let Output::Pipe(_) = output {
            self.stderr(process::Stdio::piped());
            self.stdout(process::Stdio::piped());
        }

        let mut child = self
            .spawn()
            .with_context(|_| format!("failed to spawn: {:?}", self))?;

        let threads = if let Output::Pipe(_) = output {
            let stderr = child.stderr.take().unwrap();
            let a = pipe_output(stderr, output.clone());
            let stdout = child.stdout.take().unwrap();
            let b = pipe_output(stdout, output.clone());
            Some((a, b))
        } else {
            None
        };

        let status = child
            .wait()
            .with_context(|_| format!("failed to wait on: {:?}", self))?;

        if let Some((a, b)) = threads {
            join(a);
            join(b);
        }

        if !status.success() {
            bail!(
                "command {:?} exited with unsuccessful status {:?}",
                self,
                status
            );
        }

        Ok(())
    }
}

fn pipe_output<R>(r: R, mut output: Output) -> thread::JoinHandle<()>
where
    R: 'static + Send + io::Read,
{
    thread::spawn(move || {
        let do_read = move || -> Result<()> {
            let r = io::BufReader::new(r);
            for line in r.lines() {
                let line = line?;
                output.write_all(line.as_bytes())?;
            }
            Ok(())
        };

        if let Err(e) = do_read() {
            eprintln!("Failed to pipe child output: {}", e);
        }
    })
}

fn join(handle: thread::JoinHandle<()>) {
    if let Err(_) = handle.join() {
        eprintln!("Failed to join thread");
    }
}
