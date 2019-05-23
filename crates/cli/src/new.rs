use crate::{command_ext::CommandExt, output::Output, sub_command::SubCommand, Result};
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

/// Create a new fart project.
#[derive(Clone, Debug, StructOpt)]
pub struct New {
    /// The name of the new project.
    #[structopt(parse(from_os_str))]
    name: PathBuf,

    /// The fart project template to use.
    #[structopt(default_value = "https://github.com/fitzgen/fart-template.git")]
    template: String,
}

impl SubCommand for New {
    fn run(self) -> Result<()> {
        process::Command::new("git")
            .arg("clone")
            .arg(&self.template)
            .arg(&self.name)
            .run_result(&mut Output::Inherit)?;

        process::Command::new("git")
            .args(&["remote", "remove", "origin"])
            .current_dir(&self.name)
            .run_result(&mut Output::Inherit)?;

        eprintln!(
            "\nCreated new fart project: {}",
            self.name.canonicalize()?.display()
        );
        Ok(())
    }
}
