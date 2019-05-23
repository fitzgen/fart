use crate::Result;

/// Implementations of sub-commands that can be run by the `fart` CLI tool,
/// e.g. `fart watch`.
pub trait SubCommand {
    /// Run the sub-command.
    fn run(self) -> Result<()>;

    /// Set extra arguments passed at the end. This is used to pipe through
    /// extra arguments to `cargo`. By default, they are ignored.
    fn set_extra(&mut self, extra: &[String]) {
        let _ = extra;
    }
}
