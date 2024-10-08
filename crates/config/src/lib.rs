use std::path::PathBuf;

#[cfg(feature = "clap")]
fn history() -> std::ffi::OsString {
    std::env::temp_dir()
        .join("aqua")
        .join("history.txt")
        .into_os_string()
}

#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct Config {
    #[cfg_attr(feature = "clap", clap(flatten))]
    pub repl: ReplConfig,
    #[cfg_attr(feature = "clap", clap(flatten))]
    pub compiler: CompilerConfig,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct ReplConfig {
    #[cfg_attr(feature = "clap", clap(long, default_value = history()))]
    pub history: PathBuf,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct CompilerConfig {
    /// Read source from file
    pub file: Option<PathBuf>,
    /// Loads file statement-by-statement into the REPL.
    #[cfg_attr(feature = "clap", clap(long))]
    pub interactive: bool,
    /// Print version
    #[cfg_attr(feature = "clap", clap(long))]
    pub version: bool,
    #[cfg_attr(feature = "clap", clap(subcommand))]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub enum Command {
    /// Check program for errors.
    Check,
    /// Format program.
    #[cfg_attr(feature = "clap", clap(name = "fmt"))]
    Format,
}

#[cfg(feature = "clap")]
impl Config {
    pub fn parse() -> Self {
        <Config as clap::Parser>::parse()
    }
}
