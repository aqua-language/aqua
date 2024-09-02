mod input;
mod logging;
mod version;

use anyhow::Result;
use compiler::ast::Program;
use compiler::Compiler;
use config::Command;
use config::Config;
use repl::Repl;

fn main() -> Result<()> {
    logging::init();

    let config = Config::parse();

    if config.compiler.version {
        version::print();
        return Ok(());
    }

    let mut compiler = Compiler::new(config.compiler);

    compiler.init();

    match compiler.config.command {
        Some(Command::Check) => {
            todo!()
        }
        Some(Command::Format) => {
            let (_name, source) = if let Some(path) = &compiler.config.file {
                input::read_file(path)?
            } else {
                input::read_stdin()?
            };
            match Program::parse(&source) {
                Ok(program) => println!("{}", program),
                Err(_) => print!("{}", source),
            }
            Ok(())
        }
        None => {
            if let Some(path) = &compiler.config.file {
                let (_name, source) = input::read_file(path)?;
                Repl::new(config.repl, compiler).run(Some(source))
            } else {
                Repl::new(config.repl, compiler).run(None)
            }
        }
    }
}
