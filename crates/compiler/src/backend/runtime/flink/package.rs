use anyhow::Result;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::LazyLock;

use crate::builtins::types::instance::Instance;
use crate::backend::name_generator::NAME_GENERATOR;

const POM_TEMPLATE: &str = include_str!("pom-template.xml");

pub static FLINK_WORKSPACE: LazyLock<Workspace> = LazyLock::new(Workspace::new);

#[derive(Debug)]
pub struct Workspace {
    pub(crate) root: PathBuf,
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
        let root = cache.join("workspace");
        Self { root }
    }

    pub fn clear_caches(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.root)?;
        std::fs::create_dir_all(&self.root)?;
        Ok(())
    }

    pub fn show_caches(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.root)? {
            println!("{}", entry?.path().display());
        }
        Ok(())
    }

    pub fn new_package(&self, source: impl std::fmt::Display) -> Result<Package> {
        let workspace = self.root.clone();
        let name = NAME_GENERATOR
            .lock()
            .expect("Should not be locked")
            .generate()
            .to_string();
        let path = workspace.join(&name);
        let src = path.join("src").join("main").join("java").join(&name);
        let main = src.join("Main.java");
        let pom = path.join("pom.xml");
        let target = path.join("target");
        std::fs::create_dir_all(&src)?;
        tracing::info!("Created package {}", path.display());
        let mut pom_file = File::create(&pom)?;
        let mut main_file = File::create(&main)?;
        write!(pom_file, "{}", POM_TEMPLATE.replace("{{name}}", &name))?;
        write!(main_file, "{source}")?;
        Ok(Package {
            workspace,
            target,
            path,
            main,
            pom,
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
    pub pom: PathBuf,
}

impl Package {
    pub fn compile(&self) -> Result<Executable> {
        tracing::info!(
            "Building {}",
            self.workspace.join("crates").join(&self.name).display()
        );
        let mut cmd = Command::new("mvn")
            .arg("compile")
            .arg("package")
            .arg("--file")
            .arg(&self.pom)
            .current_dir(&self.workspace)
            .stderr(Stdio::piped())
            .spawn()?;
        for line in BufReader::new(cmd.stderr.as_mut().unwrap()).lines() {
            tracing::info!("{}", line?);
        }
        if cmd.wait()?.success() {
            tracing::info!("Succeeded building crate {}", self.name);
            let path = self
                .workspace
                .join("target")
                .join(format!("{}-{}", &self.name, &self.name));
            Ok(Executable(path))
        } else {
            tracing::error!("Failed building crate {}", self.name);
            Err(anyhow::anyhow!("Build failed"))
        }
    }
}

pub struct Executable(PathBuf);

impl Executable {
    pub fn run(&self) -> Result<Instance> {
        let child = Command::new("flink")
            .arg("run")
            .arg(self.0.display().to_string())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Instance::new(child))
    }
}
