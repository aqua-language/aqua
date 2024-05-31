use std::path::PathBuf;

#[cfg(feature = "clap")]
fn history() -> std::ffi::OsString {
    std::env::temp_dir()
        .join("aqua")
        .join("history.txt")
        .into_os_string()
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct Config {
    /// Read source from file
    pub file: Option<PathBuf>,
    /// Loads file statement-by-statement into the REPL.
    #[cfg_attr(feature = "clap", clap(long))]
    pub interactive: bool,
    /// Print version
    #[cfg_attr(feature = "clap", clap(long))]
    pub version: bool,
    #[cfg_attr(feature = "clap", clap(long, default_value = history()))]
    pub history: PathBuf,
    #[cfg_attr(feature = "clap", clap(subcommand))]
    pub command: Option<Command>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub enum Command {
    /// Check but do not run the program.
    Check,
}

#[cfg(feature = "clap")]
impl Config {
    pub fn parse() -> Self {
        <Config as clap::Parser>::parse()
    }
}
