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

    if config.version {
        version::print();
        return Ok(());
    }

    let compiler = Compiler::new(config);

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
                Ok(program) => {
                    println!("{}", program);
                }
                Err(_) => {
                    print!("{}", source);
                }
            }
            Ok(())
        }
        None => {
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
    }
}
