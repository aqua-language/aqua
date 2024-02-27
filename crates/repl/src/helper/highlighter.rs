#![allow(unused)]
use std::borrow::Cow;
use std::fmt::Display;

use colored::Colorize;
use regex::Regex;
use rustyline::highlight::Highlighter;

pub struct SyntaxHighlighter {
    pub regex: Regex,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

fn join(patterns: impl IntoIterator<Item = impl Display>) -> String {
    patterns
        .into_iter()
        .map(|pattern| pattern.to_string())
        .collect::<Vec<_>>()
        .join("|")
}

fn capture_group(name: impl Display, pattern: impl Display) -> String {
    format!(r"(?P<{name}>{pattern})")
}

fn word(pattern: impl Display) -> String {
    format!(r"\b(?:{pattern})\b")
}

fn followed_by(a: impl Display, b: impl Display) -> String {
    format!(r"{a} *{b}")
}

fn maybe(a: impl Display) -> String {
    format!(r"(?:{a})?")
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let pattern = &join([
            capture_group("keyword", word(join(KEYWORDS))),
            capture_group("numeric", word(join(NUMERICS))),
            capture_group("string", join(STRINGS)),
            capture_group("builtin", word(join(BUILTINS))),
            capture_group("comment", r"#.*"),
        ]);
        Self {
            regex: Regex::new(pattern).unwrap(),
        }
    }
}

impl Highlighter for SyntaxHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        self.regex.replace_all(line, |caps: &regex::Captures| {
            if let Some(s) = caps.name("keyword") {
                s.as_str().color(KEYWORD_COLOR).bold().to_string()
            } else if let Some(s) = caps.name("numeric") {
                s.as_str().color(NUMERIC_COLOR).to_string()
            } else if let Some(s) = caps.name("string") {
                s.as_str().color(STRING_COLOR).to_string()
            } else if let Some(s) = caps.name("builtin") {
                s.as_str().color(BUILTIN_COLOR).bold().to_string()
            } else if let Some(s) = caps.name("comment") {
                s.as_str().color(COMMENT_COLOR).to_string()
            } else {
                unreachable!()
            }
        })
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.regex.is_match(line)
    }
}

pub const KEYWORDS: &[&str] = &[
    "mod", "and", "as", "break", "continue", "def", "desc", "else", "for", "from", "fun", "group",
    "if", "in", "into", "join", "loop", "match", "not", "on", "or", "of", "return", "select",
    "compute", "type", "val", "var", "where", "with", "while", "use", "union", "over", "roll",
    "order", "enum",
];

pub const NUMERICS: &[&str] = &[
    "0[bB][0-1][_0-1]*",
    "0[oO][0-7][_0-7]*",
    "[0-9][_0-9]*",
    "0[xX][0-9a-fA-F][_0-9a-fA-F]*",
];

pub const BUILTINS: &[&str] = &["true", "false"];

pub const STRINGS: &[&str] = &[r#""([^"\\]|\\.)*""#, r"'[^']'"];

pub const COMMENTS: &[&str] = &["#[^\r\n]*", "#[^\n]*"];

pub const TYPES: &[&str] = &[
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "f32",
    "f64",
    "Int",
    "Float",
    "bool",
    "char",
    "String",
    "Vec",
    "Option",
    "Result",
    "Set",
    "Dict",
    "Stream",
    "Matrix",
    "File",
    "SocketAddr",
    "Url",
    "Path",
    "Duration",
    "Time",
];

use colored::Color;
use colored::Color::TrueColor;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::TrueColor { r, g, b }
}

pub const KEYWORD_COLOR: Color = rgb(0, 95, 135);
pub const MACRO_COLOR: Color = rgb(95, 135, 0);
pub const VAR_COLOR: Color = rgb(255, 0, 0);
pub const VAL_COLOR: Color = rgb(68, 68, 68);
pub const TYPE_COLOR: Color = rgb(0, 135, 0);
pub const DEF_COLOR: Color = rgb(0, 135, 175);
pub const NUMERIC_COLOR: Color = rgb(215, 95, 0);
pub const VARIANT_COLOR: Color = rgb(0, 135, 0);
pub const STRING_COLOR: Color = rgb(95, 135, 0);
pub const BUILTIN_COLOR: Color = rgb(0, 135, 0);
pub const COMMENT_COLOR: Color = rgb(135, 135, 135);
