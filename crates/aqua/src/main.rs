mod input;
mod logging;
mod version;

use anyhow::Result;
use compiler::Compiler;
use config::Config;
use repl::Repl;

fn main() -> Result<()> {
    logging::init();

    let config = Config::parse();

    if config.version {
        version::print();
        return Ok(());
    }

    let compiler = Compiler::new(config);

    if let Some(path) = &compiler.config.file {
        let (_name, source) = input::read_file(path)?;
        if compiler.config.interactive {
            Repl::new(compiler).run(Some(source))
        } else {
            todo!()
        }
    } else {
        Repl::new(compiler).run(None)
    }
}
