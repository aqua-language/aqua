use crate::sources::Sources;
use crate::span::Span;
use std::io::Write;

use ariadne::Config;
use ariadne::Label;
use ariadne::ReportKind;

#[macro_export]
macro_rules! loc {
    () => {{
        #[cfg(debug_assertions)]
        {
            std::panic::Location::caller()
        }
        #[cfg(not(debug_assertions))]
        {
            ""
        }
    }};
}

#[derive(Debug, Default)]
pub struct Report {
    diags: Vec<ariadne::Report<'static, Span>>,
    pub supress: bool,
}

impl Report {
    pub fn new() -> Self {
        Self {
            diags: Vec::new(),
            supress: false,
        }
    }

    pub fn merge(&mut self, other: &mut Report) {
        self.diags.append(&mut other.diags);
    }

    pub fn is_empty(&self) -> bool {
        self.diags.is_empty()
    }

    pub fn err(&mut self, span: Span, label: impl AsRef<str>, msg: impl AsRef<str>) {
        if !self.supress {
            let kind = ReportKind::Error;
            let report = ariadne::Report::build(kind, *span.file(), *span.start() as usize)
                .with_label(Label::new(span).with_message(msg.as_ref()))
                .with_message(label.as_ref())
                .with_config(Config::default().with_color(false))
                .finish();
            self.diags.push(report);
        }
    }

    pub fn err2(
        &mut self,
        s0: Span,
        s1: Span,
        label: impl AsRef<str>,
        msg0: impl AsRef<str>,
        msg1: impl AsRef<str>,
    ) {
        let kind = ReportKind::Error;
        let report = ariadne::Report::build(kind, *s0.file(), *s0.start() as usize)
            .with_label(Label::new(s0).with_message(msg0.as_ref()))
            .with_label(Label::new(s1).with_message(msg1.as_ref()))
            .with_message(label.as_ref())
            .with_config(Config::default().with_color(false))
            .finish();
        self.diags.push(report);
    }

    pub fn print(&mut self, sources: &mut Sources) -> std::io::Result<()> {
        for diag in self.diags.drain(..) {
            diag.eprint(&mut *sources)?;
        }
        Ok(())
    }

    pub fn string(&mut self, sources: &mut Sources) -> Result<String, std::string::FromUtf8Error> {
        let mut buf = Vec::new();
        for diag in self.diags.drain(..) {
            diag.write(&mut *sources, &mut buf).unwrap();
            writeln!(&mut buf).unwrap();
        }
        String::from_utf8(buf)
    }
}
