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
        Some(Command::Run) => {
            let (name, source) = if let Some(path) = &compiler.config.file {
                input::read_file(path)?
            } else {
                input::read_stdin()?
            };
            compiler.compile_and_run(name, &source)
        }
        None => {
            if let Some(path) = &compiler.config.file {
                let (name, source) = input::read_file(path)?;
                if compiler.config.interactive {
                    Repl::new(config.repl, compiler).run(Some(source))
                } else {
                    compiler.compile_and_run(name, &source)?;
                    Repl::new(config.repl, compiler).run(None)
                }
            } else {
                Repl::new(config.repl, compiler).run(None)
            }
        }
    }
}
