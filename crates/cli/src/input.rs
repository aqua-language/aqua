use anyhow::Result;
use std::io::Read;
use std::path::Path;

pub fn read_file(path: &Path) -> Result<(String, String)> {
    let name = path.display().to_string();
    let source = std::fs::read_to_string(path)?;
    if path.is_absolute() {
        std::env::set_current_dir(path.parent().unwrap())?;
    } else {
        let mut relative_path = std::env::current_dir()?;
        relative_path.push(path);
        std::env::set_current_dir(relative_path.parent().unwrap())?;
    }
    tracing::info!("Updated cwd to: {}", std::env::current_dir()?.display());
    Ok((name, source))
}

pub fn read_stdin() -> Result<(String, String)> {
    let mut source = String::new();
    let mut stdin = std::io::stdin();
    stdin.read_to_string(&mut source)?;
    Ok(("stdin".to_string(), source))
}
