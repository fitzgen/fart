use crate::{cargo, git, sub_command::SubCommand, Result};
use failure::ResultExt;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

/// Run a fart project and generate a new SVG.
#[derive(Clone, Debug, StructOpt)]
pub struct Run {
    /// The fart project to run.
    #[structopt(parse(from_os_str), default_value = ".")]
    project: PathBuf,

    /// Extra arguments passed along to `cargo run`.
    #[structopt(long = "")]
    extra: Vec<String>,
}

impl Run {
    pub fn new(project: PathBuf, extra: Vec<String>) -> Run {
        Run { project, extra }
    }
}

impl SubCommand for Run {
    fn set_extra(&mut self, extra: &[String]) {
        assert!(self.extra.is_empty());
        self.extra = extra.iter().cloned().collect();
    }

    fn run(&mut self) -> Result<()> {
        let now = chrono::Utc::now();
        let now = now.format("%Y-%m-%d-%H-%M-%S-%f").to_string();

        let images = self.project.join("images");
        fs::create_dir_all(&images)
            .with_context(|_| format!("failed to create directory: {}", images.display()))?;

        let mut file_name = images.join(&now);
        file_name.set_extension("svg");
        let file_name = file_name.canonicalize().unwrap_or(file_name);

        cargo::build(&self.project, &self.extra)?;

        cargo::run(
            &self.project,
            &self.extra,
            vec![("FART_FILE_NAME", &file_name)],
        )?;

        link_as_latest(&self.project, &file_name)?;

        git::add_all(&self.project)?;
        git::commit(&self.project, &now)?;
        Ok(())
    }
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
