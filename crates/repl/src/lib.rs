#![allow(unused)]

use compiler::ast::Ast;
use compiler::syntax::parser::Parser;
use compiler::Compiler;
use config::ReplConfig;
use helper::validator::StmtIterator;
use rustyline::Movement;

use std::io;
use std::io::LineWriter;
use std::io::Write;
use std::process::exit;
use std::rc::Rc;

use anyhow::Result;
use colored::Color;
use colored::Color::Blue;
use colored::Color::Green;
use colored::Color::Red;
use colored::Colorize;
use rustyline::completion::FilenameCompleter;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::Cmd;
use rustyline::CompletionType;
use rustyline::EditMode;
use rustyline::Editor;
use rustyline::EventHandler;

pub mod helper;
use crate::helper::Helper;

use rustyline::history::MemHistory;
use rustyline::KeyCode;
use rustyline::KeyEvent;
use rustyline::Modifiers;

#[derive(Debug)]
pub struct Repl {
    pub(crate) count: usize,
    pub(crate) editor: Editor<Helper, FileHistory>,
    pub compiler: Compiler,
    pub(crate) config: ReplConfig,
}

impl Drop for Repl {
    fn drop(&mut self) {
        self.editor.save_history(&self.config.history).ok();
    }
}

impl Repl {
    pub fn new(config: ReplConfig, compiler: Compiler) -> Self {
        let mut editor = Self::editor();
        if !config.history.exists() {
            std::fs::create_dir_all(config.history.parent().unwrap())
                .expect("Unable to create history directory");
            std::fs::File::create(&config.history).expect("Unable to create history");
        }
        editor.load_history(&config.history).ok();
        Self {
            count: 0,
            editor,
            compiler,
            config,
        }
    }

    fn editor() -> Editor<Helper, FileHistory> {
        let mut editor = Editor::new().expect("Unable to create editor");
        editor.set_helper(Some(Helper::default()));
        editor
            .set_history_ignore_dups(true)
            .expect("Unable to set history ignore dups");
        editor.set_edit_mode(EditMode::Vi);
        editor.set_completion_type(CompletionType::List);
        editor.bind_sequence(KeyEvent::ctrl('j'), Cmd::NextHistory);
        editor.bind_sequence(KeyEvent::ctrl('k'), Cmd::PreviousHistory);
        editor.bind_sequence(KeyEvent::ctrl('l'), Cmd::ClearScreen);
        editor.bind_sequence(KeyEvent::ctrl('c'), Cmd::Interrupt);
        editor.bind_sequence(KeyEvent::ctrl('v'), Cmd::YankPop);
        editor.bind_sequence(KeyEvent::ctrl('e'), Cmd::Move(Movement::EndOfLine));
        editor.bind_sequence(KeyEvent::ctrl('a'), Cmd::Move(Movement::BeginningOfLine));
        editor.bind_sequence(
            KeyEvent::ctrl('M'),
            Cmd::AcceptOrInsertLine {
                accept_in_the_middle: false,
            },
        );
        editor.bind_sequence(
            KeyEvent(KeyCode::Enter, Modifiers::CTRL),
            Cmd::HistorySearchForward,
        );
        editor
    }

    pub(crate) fn color(&mut self, color: Color) {
        self.editor.helper_mut().unwrap().prompt_color = color;
    }

    pub(crate) fn readline(&mut self) -> std::result::Result<String, ReadlineError> {
        self.count += 1;
        self.editor.readline(">> ")
    }

    pub fn run(&mut self, initial: Option<String>) -> Result<()> {
        self.color(Color::Green);
        let mut stmts = Some(initial.iter().flat_map(|s| StmtIterator::new(s)));
        loop {
            let input = stmts
                .as_mut()
                .and_then(Iterator::next)
                .map(|stmt| self.editor.readline_with_initial(">> ", (stmt, "")))
                .unwrap_or_else(|| self.editor.readline(">> "));
            match input {
                Ok(input) => {
                    let input: Rc<str> = Rc::from(input);
                    self.editor.add_history_entry(input.as_ref());
                    self.compiler.run(self.count, input.as_ref());
                    if self.compiler.report.is_empty() {
                        self.color(Green);
                    } else {
                        self.compiler.report.print(&mut self.compiler.sources)?;
                        self.color(Red);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    stmts = None;
                    eprintln!("Interrupted");
                    self.color(Red);
                }
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        Ok(())
    }
}
