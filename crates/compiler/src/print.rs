use crate::ast::Index;
use crate::ast::Name;
use crate::lexer::Token;
use crate::symbol::Symbol;

pub trait Print<'b> {
    // Required methods
    fn fmt(&mut self) -> &mut std::fmt::Formatter<'b>;
    fn get_indent(&mut self) -> &mut usize;
    fn should_indent(&mut self) -> bool;

    // Default methods
    fn tab(&mut self) -> std::fmt::Result {
        for _ in 0..*self.get_indent() {
            write!(self.fmt(), "    ")?;
        }
        Ok(())
    }

    fn kw(&mut self, s: &str) -> std::fmt::Result {
        write!(self.fmt(), "{}", s)
    }

    fn space(&mut self) -> std::fmt::Result {
        write!(self.fmt(), " ")
    }

    fn punct(&mut self, s: &str) -> std::fmt::Result {
        write!(self.fmt(), "{}", s)
    }

    fn token(&mut self, t: &Token) -> std::fmt::Result {
        write!(self.fmt(), "{t}")
    }

    fn newline(&mut self) -> std::fmt::Result {
        if !self.should_indent() {
            writeln!(self.fmt())?;
        }
        self.tab()
    }

    fn sep<'c, T: 'c>(
        &mut self,
        sep: &str,
        space: bool,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> std::fmt::Result,
    ) -> std::fmt::Result {
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
        f: impl Fn(&mut Self, &'c T) -> std::fmt::Result,
    ) -> std::fmt::Result {
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
        f: impl Fn(&mut Self, &'c T) -> std::fmt::Result,
    ) -> std::fmt::Result {
        self.sep(",", true, iter, f)
    }

    fn comma_sep_trailing<'c, T: 'c>(
        &mut self,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> std::fmt::Result,
    ) -> std::fmt::Result {
        self.sep_trailing(",", iter, f)
    }

    fn newline_comma_sep<'c, T: 'c>(
        &mut self,
        iter: impl IntoIterator<Item = &'c T>,
        f: impl Fn(&mut Self, &'c T) -> std::fmt::Result,
    ) -> std::fmt::Result {
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
        f: impl Fn(&mut Self, &'c T) -> std::fmt::Result,
    ) -> std::fmt::Result {
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

    fn group(
        &mut self,
        f: impl Fn(&mut Self) -> std::fmt::Result,
        l: &str,
        r: &str,
    ) -> std::fmt::Result {
        self.punct(l)?;
        f(self)?;
        self.punct(r)?;
        Ok(())
    }

    fn indented(&mut self, f: impl Fn(&mut Self) -> std::fmt::Result) -> std::fmt::Result {
        *self.get_indent() += 1;
        f(self)?;
        *self.get_indent() -= 1;
        Ok(())
    }

    fn brace(&mut self, fun: impl Fn(&mut Self) -> std::fmt::Result) -> std::fmt::Result {
        self.group(fun, "{", "}")
    }

    fn paren(&mut self, fun: impl Fn(&mut Self) -> std::fmt::Result) -> std::fmt::Result {
        self.group(fun, "(", ")")
    }

    fn brack(&mut self, fun: impl Fn(&mut Self) -> std::fmt::Result) -> std::fmt::Result {
        self.group(fun, "[", "]")
    }

    fn if_nonempty<T>(
        &mut self,
        items: &[T],
        f: impl Fn(&mut Self, &[T]) -> std::fmt::Result,
    ) -> std::fmt::Result {
        if items.is_empty() {
            Ok(())
        } else {
            f(self, items)
        }
    }

    fn if_some<T>(
        &mut self,
        item: &Option<T>,
        f: impl Fn(&mut Self, &T) -> std::fmt::Result,
    ) -> std::fmt::Result {
        if let Some(item) = item {
            f(self, item)
        } else {
            Ok(())
        }
    }

    fn name(&mut self, s: &Name) -> std::fmt::Result {
        write!(self.fmt(), "{}", &s.data)
    }

    fn index(&mut self, i: &Index) -> std::fmt::Result {
        write!(self.fmt(), "{}", i.data)
    }

    fn lit(&mut self, s: impl std::fmt::Display) -> std::fmt::Result {
        write!(self.fmt(), "{}", s)
    }

    fn str(&mut self, s: &Symbol) -> std::fmt::Result {
        self.punct("\"")?;
        self.lit(s)?;
        self.punct("\"")
    }

    fn char(&mut self, c: char) -> std::fmt::Result {
        self.punct("'")?;
        self.lit(c)?;
        self.punct("'")
    }

    fn scope<T>(
        &mut self,
        items: &[T],
        f: impl Fn(&mut Self, &T) -> std::fmt::Result,
    ) -> std::fmt::Result {
        self.brace(|this| {
            if items.is_empty() {
                Ok(())
            } else {
                this.indented(|this| this.newline_sep(items, |this, item| f(this, item)))?;
                this.newline()
            }
        })
    }

    fn comma_scope<T>(
        &mut self,
        items: &[T],
        f: impl Fn(&mut Self, &T) -> std::fmt::Result,
    ) -> std::fmt::Result {
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
