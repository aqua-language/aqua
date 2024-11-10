mod input;
mod logging;
mod version;

use anyhow::Result;
use colored::Color;
use colored::Colorize;
use compiler::ast::Program;
use compiler::Compiler;
use config::Command;
use config::CompilerConfig;
use config::Config;
use config::InspectMode;
use repl::Repl;

fn main() -> Result<()> {
    logging::init();

    let config = Config::parse();

    if config.compiler.version {
        version::print();
        return Ok(());
    }

    let mut compiler = Compiler::new(config.compiler);

    match &compiler.config.command {
        Some(Command::Check(_)) => {
            let (name, source) = read(&compiler.config)?;
            message("Checking", format_args!("{name}"));
            let time = std::time::Instant::now();
            compiler.init();
            let result = compiler.check(name, &source);
            let duration = time.elapsed().as_millis() as f64 / 1000.0;
            match result {
                Ok(_) => message("Finished", format_args!("in {duration}s")),
                Err(_) => {
                    let n = compiler.report.len();
                    compiler.print_report();
                    message(
                        "Failure:",
                        format_args!("could not compile due to {n} previous errors."),
                    );
                }
            }
            Ok(())
        }
        Some(Command::Format(_)) => {
            let (_, source) = read(&compiler.config)?;
            match Program::parse(&source) {
                Ok(program) => println!("{}", program),
                Err(_) => print!("{}", source),
            }
            Ok(())
        }
        Some(Command::Run(_)) => {
            let (name, source) = read(&compiler.config)?;
            compiler.init();
            compiler.run(name, &source)
        }
        Some(Command::Inspect(cmd)) => {
            let (name, source) = read(&compiler.config)?;
            match cmd.mode {
                InspectMode::Desugar => match compiler.querycomp(&name, &source) {
                    Ok(program) => println!("{}", program),
                    Err(_) => print!("{}", source),
                },
                InspectMode::Type => {
                    compiler.init();
                    match compiler.infer(&name, &source) {
                        Ok(program) => println!("{}", program.verbose()),
                        Err(_) => print!("{}", source),
                    }
                }
            }
            Ok(())
        }
        Some(Command::Lsp) => {
            if let Err(e) = lsp::start() {
                eprintln!("Error: {}", e);
            }
            Ok(())
        }
        None => {
            compiler.init();
            let mut repl = Repl::new(config.repl, compiler);
            if let Some(path) = &repl.compiler.config.file() {
                let (name, source) = input::read_file(path)?;
                if repl.compiler.config.interactive {
                    repl.run(Some(source))
                } else {
                    repl.compiler.run(name, &source)?;
                    repl.run(None)
                }
            } else {
                repl.run(None)
            }
        }
    }
}

fn message(label: &str, content: std::fmt::Arguments) {
    let pad = 12;
    let label = label.color(Color::BrightBlue).bold();
    println!("{:>pad$} {content}", label);
}

fn read(config: &CompilerConfig) -> Result<(String, String)> {
    if let Some(path) = &config.file() {
        input::read_file(path)
    } else {
        input::read_stdin()
    }
}
