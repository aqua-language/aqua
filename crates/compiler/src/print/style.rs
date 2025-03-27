use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

pub struct Style<T> {
    data: T,
    color: Color,
    bold: bool,
    italic: bool,
    underline: bool,
}

#[allow(unused)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Orange,
}

impl Color {
    fn ansi_code(&self) -> &'static str {
        match self {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::Orange => "31;43",
        }
    }
}

impl<T> Style<T> {
    pub fn new(data: T, color: Color) -> Self {
        Self {
            data,
            color,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[allow(unused)]
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    #[allow(unused)]
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

impl<T> Display for Style<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\x1b[")?;
        if self.bold {
            write!(f, "1;")?;
        }
        if self.italic {
            write!(f, "3;")?;
        }
        if self.underline {
            write!(f, "4;")?;
        }
        write!(f, "{}m", self.color.ansi_code())?;
        write!(f, "{}", self.data)?;
        write!(f, "\x1b[0m")
    }
}

pub trait Styled {
    fn color(&self, color: Color) -> Style<&Self> {
        Style::new(self, color)
    }
}

impl<T> Styled for T {}
