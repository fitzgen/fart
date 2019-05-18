use crate::Result;
use failure::{bail, ResultExt};
use std::process;

/// Extension trait for `std::process::Command`.
pub trait CommandExt {
    /// Run the command and get a result based on if it completed successfully
    /// or not.
    fn run_result(self) -> Result<()>;
}

impl CommandExt for &'_ mut process::Command {
    fn run_result(self) -> Result<()> {
        let status = self
            .status()
            .with_context(|_| format!("failed to execute: {:?}", self))?;
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
