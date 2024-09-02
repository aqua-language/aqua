use anyhow::Result;
use names::Generator;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;

use crate::builtins::types::instance::Instance;

const RUNTIME: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../runtime");

pub struct Workspace {
    pub(crate) name_generator: Generator<'static>,
    pub(crate) path: PathBuf,
    pub(crate) crates: PathBuf,
    pub(crate) target: PathBuf,
    pub pids: Vec<u32>,
}

impl std::fmt::Debug for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("workspace", &self.path)
            .field("crates", &self.crates)
            .finish()
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        let dir = directories::ProjectDirs::from("org", "aqua", "rust").unwrap();
        let cache = dir.cache_dir();
        tracing::info!("Cache directory: {}", cache.display());
        let path = cache.join("workspace");
        let toml = path.join("Cargo.toml");
        let crates = path.join("crates");
        let target = path.join("target");
        std::fs::create_dir_all(&crates).unwrap();
        std::fs::write(
            &toml,
            indoc::formatdoc!(
                r#"[workspace]
                   members = [ "crates/*" ]
                   resolver = "2"

                   [workspace.package]
                   version = "0.0.0"
                   edition = "2024"

                   [workspace.dependencies]
                   runtime = {{ path = "{RUNTIME}" }}"#,
            ),
        )
        .expect("Unable to write file");
        Self {
            name_generator: Generator::new(names::ADJECTIVES, names::NOUNS, names::Name::Plain),
            path,
            crates,
            target,
            pids: Vec::new(),
        }
    }

    pub fn clear_caches(&mut self) -> Result<()> {
        std::fs::remove_dir_all(&self.crates)?;
        std::fs::create_dir_all(&self.crates)?;
        std::fs::remove_dir_all(&self.target)?;
        Ok(())
    }

    pub fn show_caches(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.crates)? {
            println!("{}", entry?.path().display());
        }
        Ok(())
    }

    pub fn new_package(&mut self, source: impl std::fmt::Display) -> Result<Package> {
        let base = self.name_generator.next().expect("Should have a name");
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let workspace = self.path.clone();
        let target = self.target.clone();
        let name = format!("{base}-{time}");
        let path = self.crates.join(&name);
        let src = path.join("src");
        let main = src.join("main.rs");
        let toml = path.join("Cargo.toml");
        std::fs::create_dir_all(&src)?;
        std::fs::write(
            &toml,
            indoc::formatdoc!(
                r#"[package]
                   name = "{name}"
                   version.workspace = true
                   edition.workspace = true

                   [dependencies]
                   runtime.workspace = true"#,
            ),
        )?;
        let mut file = File::create("output.txt")?;
        write!(file, "{source}")?;
        Ok(Package {
            workspace,
            target,
            path,
            main,
            toml,
            name,
        })
    }
}

pub struct Package {
    pub workspace: PathBuf,
    pub target: PathBuf,
    pub name: String,
    pub path: PathBuf,
    pub main: PathBuf,
    pub toml: PathBuf,
}

impl Package {
    pub fn compile(&self) -> Result<Executable> {
        tracing::info!(
            "Building {}",
            self.workspace.join("crates").join(&self.name).display()
        );
        let mut cmd = Command::new("cargo")
            .arg("build")
            .arg("--package")
            .arg(&self.name)
            .arg("--release")
            .arg("--target-dir")
            .arg(&self.workspace.join("target"))
            .current_dir(&self.workspace)
            .stderr(Stdio::piped())
            .spawn()?;
        for line in BufReader::new(cmd.stderr.as_mut().unwrap()).lines() {
            tracing::info!("{}", line?);
        }
        if cmd.wait()?.success() {
            tracing::info!("Succeeded building crate {}", self.name);
            let path = self.workspace.join("target/release").join(&self.name);
            Ok(Executable { path })
        } else {
            tracing::error!("Failed building crate {}", self.name);
            Err(anyhow::anyhow!("Build failed"))
        }
    }
}

pub struct Executable {
    pub path: PathBuf,
}

impl Executable {
    pub fn debug(&self) -> Result<()> {
        tracing::info!(
            "Running `{}` from `{}`",
            self.path.display(),
            std::env::current_dir()?.display()
        );
        Ok(())
    }

    // Run locally.
    pub fn run_locally(&self) -> Result<Instance> {
        self.debug()?;
        let child = Command::new(&self.path)
            .current_dir(std::env::current_dir()?)
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Instance {
            child: Rc::new(RefCell::new(child)),
        })
    }

    // Run remotely using SSH
    pub fn run_remotely(&self, ip: IpAddr) -> Result<Instance> {
        self.debug()?;
        let child = Command::new("ssh")
            .arg(ip.to_string())
            .arg(self.path.display().to_string())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Instance {
            child: Rc::new(RefCell::new(child)),
        })
    }
}
