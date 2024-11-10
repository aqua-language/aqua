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
    /// Loads file statement-by-statement into the REPL.
    #[cfg_attr(feature = "clap", clap(long))]
    pub interactive: bool,
    /// Print version.
    #[cfg_attr(feature = "clap", clap(long))]
    pub version: bool,
    #[cfg_attr(feature = "clap", clap(subcommand))]
    pub command: Option<Command>,
}

impl CompilerConfig {
    pub fn file(&self) -> Option<&PathBuf> {
        self.command.as_ref().and_then(|c| match c {
            Command::Check(sub) => sub.file.as_ref(),
            Command::Format(sub) => sub.file.as_ref(),
            Command::Run(sub) => sub.file.as_ref(),
            Command::Inspect(sub) => sub.file.as_ref(),
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub enum Command {
    /// Check program for errors.
    Check(Check),
    /// Format program.
    #[cfg_attr(feature = "clap", clap(name = "fmt"))]
    Format(Format),
    /// Run program.
    Run(Run),
    /// Inspect intermediate AST.
    Inspect(Inspect),
    /// Start language-server.
    Lsp,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct Check {
    /// Read source from file.
    pub file: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct Format {
    /// Read source from file.
    #[cfg_attr(feature = "clap", clap(long))]
    pub desugared: bool,
    /// Read source from file.
    #[cfg_attr(feature = "clap", clap(long))]
    pub typed: bool,
    /// Read source from file.
    pub file: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct Run {
    /// Read source from file.
    pub file: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
pub struct Inspect {
    #[cfg_attr(feature = "clap", clap(long))]
    pub mode: InspectMode,
    /// Read source from file.
    pub file: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum InspectMode {
    #[default]
    Desugar,
    Type,
}

#[cfg(feature = "clap")]
impl Config {
    pub fn parse() -> Self {
        <Config as clap::Parser>::parse()
    }
}
