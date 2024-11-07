use crate::source::Cache;
use crate::span::Span;
use std::io::Write;

use ariadne::Config;
use ariadne::Label;
use ariadne::ReportKind;

#[derive(Debug, Default)]
pub struct Report {
    pub diags: Vec<Diagnostic>,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub label: Message,
    pub messages: Vec<Message>,
}

#[derive(Debug)]
pub struct Message {
    pub span: Span,
    pub text: String,
}

impl Message {
    pub fn new(span: Span, text: String) -> Self {
        Self { span, text }
    }
}

impl From<Diagnostic> for ariadne::Report<'static, Span> {
    fn from(diag: Diagnostic) -> Self {
        let span = diag.label.span;
        let kind = ReportKind::Error;
        let mut report = ariadne::Report::build(kind, span)
            .with_message(diag.label.text)
            .with_config(Config::default().with_color(false));
        for msg in diag.messages {
            report = report.with_label(Label::new(span).with_message(msg.text));
        }
        report.finish()
    }
}

impl Report {
    pub fn new() -> Self {
        Self { diags: Vec::new() }
    }

    pub fn append(&mut self, other: &mut Report) {
        self.diags.append(&mut other.diags);
    }

    pub fn is_empty(&self) -> bool {
        self.diags.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diags.len()
    }

    pub fn err(&mut self, span: Span, label: impl AsRef<str>, msg: impl AsRef<str>) {
        let diagnostic = Diagnostic {
            label: Message::new(span, label.as_ref().to_string()),
            messages: vec![Message::new(span, msg.as_ref().to_string())],
        };
        self.diags.push(diagnostic);
    }

    pub fn err2(
        &mut self,
        span0: Span,
        span1: Span,
        label: impl AsRef<str>,
        msg0: impl AsRef<str>,
        msg1: impl AsRef<str>,
    ) {
        let diagnostic = Diagnostic {
            label: Message::new(span0, label.as_ref().to_string()),
            messages: vec![
                Message::new(span0, msg0.as_ref().to_string()),
                Message::new(span1, msg1.as_ref().to_string()),
            ],
        };
        self.diags.push(diagnostic);
    }

    pub fn print(&mut self, sources: &mut Cache) -> std::io::Result<()> {
        for diag in self.diags.drain(..) {
            let report: ariadne::Report<Span> = diag.into();
            report.eprint(&mut *sources)?;
        }
        Ok(())
    }

    pub fn string(&mut self, sources: &mut Cache) -> Result<String, std::string::FromUtf8Error> {
        let mut buf = Vec::new();
        for diag in self.diags.drain(..) {
            let report: ariadne::Report<Span> = diag.into();
            report.write(&mut &mut *sources, &mut buf).unwrap();
            writeln!(&mut buf).unwrap();
        }
        String::from_utf8(buf)
    }
}
