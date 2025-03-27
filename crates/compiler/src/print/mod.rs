mod style;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use style::Color;
use style::Styled;

use crate::ast::Index;
use crate::ast::Name;

pub trait Print<'b> {
    fn formatter_mut(&mut self) -> &mut Formatter<'b>;
    fn indent_mut(&mut self) -> &mut usize;

    fn tab(&mut self) -> Result {
        for _ in 0..*self.indent_mut() {
            write!(self.formatter_mut(), "    ")?;
        }
        Ok(())
    }

    fn keyword(&mut self, s: impl Display) -> Result {
        if cfg!(test) {
            write!(self.formatter_mut(), "{s}")
        } else {
            write!(self.formatter_mut(), "{}", s.color(Color::Red).bold())
        }
    }

    fn string(&mut self, s: impl Display) -> Result {
        if cfg!(test) {
            write!(self.formatter_mut(), "\"{s}\"")
        } else {
            write!(
                self.formatter_mut(),
                "{}",
                format_args!("\"{s}\"").color(Color::Green)
            )
        }
    }

    fn char(&mut self, s: impl Display) -> Result {
        if cfg!(test) {
            write!(self.formatter_mut(), "'{s}'")
        } else {
            write!(
                self.formatter_mut(),
                "{}",
                format_args!("'{s}'").color(Color::Green)
            )
        }
    }

    fn numeric(&mut self, s: impl Display) -> Result {
        if cfg!(test) {
            write!(self.formatter_mut(), "{s}")
        } else {
            write!(self.formatter_mut(), "{}", s.color(Color::Orange))
        }
    }

    fn space(&mut self) -> Result {
        write!(self.formatter_mut(), " ")
    }

    fn punct(&mut self, s: impl Display) -> Result {
        write!(self.formatter_mut(), "{}", s)
    }

    fn newline(&mut self) -> Result {
        writeln!(self.formatter_mut())?;
        self.tab()
    }

    fn sep<'c, T: 'c>(
        &mut self,
        sep: &str,
        space: bool,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> Result,
    ) -> Result {
        let mut iter = iter.into_iter();
        if let Some(x) = iter.next() {
            f(self, x)?;
            for x in iter {
                self.punct(sep)?;
                if space {
                    self.space()?;
                }
                f(self, x)?;
            }
        }
        Ok(())
    }

    fn sep_trailing<'c, T: 'c>(
        &mut self,
        sep: &str,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> Result,
    ) -> Result {
        let mut iter = iter.into_iter();
        if let Some(x) = iter.next() {
            f(self, x)?;
            self.punct(sep)?;
            if let Some(x) = iter.next() {
                self.space()?;
                f(self, x)?;
                for x in iter {
                    self.punct(sep)?;
                    self.space()?;
                    f(self, x)?;
                }
            }
        }
        Ok(())
    }

    fn comma_sep<'c, T: 'c>(
        &mut self,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> Result,
    ) -> Result {
        self.sep(",", true, iter, f)
    }

    fn comma_sep_trailing<'c, T: 'c>(
        &mut self,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> Result,
    ) -> Result {
        self.sep_trailing(",", iter, f)
    }

    fn newline_comma_sep<'c, T: 'c>(
        &mut self,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> Result,
    ) -> Result {
        let mut iter = iter.into_iter();
        if let Some(x) = iter.next() {
            self.newline()?;
            f(self, x)?;
            self.punct(",")?;
            for x in iter {
                self.newline()?;
                f(self, x)?;
                self.punct(",")?;
            }
        }
        Ok(())
    }

    fn newline_sep<'c, T: 'c>(
        &mut self,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> Result,
    ) -> Result {
        let mut iter = iter.into_iter();
        if let Some(x) = iter.next() {
            f(self, x)?;
            for x in iter {
                self.newline()?;
                f(self, x)?;
            }
        }
        Ok(())
    }

    fn group(&mut self, f: impl Fn(&mut Self) -> Result, l: &str, r: &str) -> Result {
        self.punct(l)?;
        f(self)?;
        self.punct(r)?;
        Ok(())
    }

    fn indented(&mut self, f: impl Fn(&mut Self) -> Result) -> Result {
        *self.indent_mut() += 1;
        f(self)?;
        *self.indent_mut() -= 1;
        Ok(())
    }

    fn brace(&mut self, fun: impl Fn(&mut Self) -> Result) -> Result {
        self.group(fun, "{", "}")
    }

    fn paren(&mut self, fun: impl Fn(&mut Self) -> Result) -> Result {
        self.group(fun, "(", ")")
    }

    fn brack(&mut self, fun: impl Fn(&mut Self) -> Result) -> Result {
        self.group(fun, "[", "]")
    }

    fn bars(&mut self, fun: impl Fn(&mut Self) -> Result) -> Result {
        self.group(fun, "|", "|")
    }

    fn angle(&mut self, fun: impl Fn(&mut Self) -> Result) -> Result {
        self.group(fun, "<", ">")
    }

    fn if_nonempty<T>(&mut self, items: &[T], f: impl Fn(&mut Self, &[T]) -> Result) -> Result {
        if items.is_empty() {
            Ok(())
        } else {
            f(self, items)
        }
    }

    fn if_some<T>(&mut self, item: &Option<T>, f: impl Fn(&mut Self, &T) -> Result) -> Result {
        if let Some(item) = item {
            f(self, item)
        } else {
            Ok(())
        }
    }

    fn name(&mut self, s: &Name) -> Result {
        write!(self.formatter_mut(), "{}", &s.data)
    }

    fn index(&mut self, i: &Index) -> Result {
        write!(self.formatter_mut(), "{}", i.data)
    }

    fn lit(&mut self, s: impl Display) -> Result {
        write!(self.formatter_mut(), "{}", s)
    }

    fn scope<T>(&mut self, items: &[T], f: impl Fn(&mut Self, &T) -> Result) -> Result {
        self.brace(|this| {
            if items.is_empty() {
                Ok(())
            } else {
                this.indented(|this| this.newline_sep(items, |this, item| f(this, item)))?;
                this.newline()
            }
        })
    }

    fn comma_scope<T>(&mut self, items: &[T], f: impl Fn(&mut Self, &T) -> Result) -> Result {
        self.brace(|this| {
            if items.is_empty() {
                Ok(())
            } else {
                this.indented(|this| this.newline_comma_sep(items, &f))?;
                this.newline()
            }
        })
    }
}
