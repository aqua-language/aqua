use crate::syntax::source::Cache;
use crate::syntax::source::SourceId;
use crate::syntax::span::Span;
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

impl ariadne::Span for Span {
    fn start(&self) -> usize {
        self.start().unwrap() as usize
    }

    fn end(&self) -> usize {
        self.end().unwrap() as usize
    }

    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        match self {
            Span::Source(id, _, _) => id,
            Span::Generated => unreachable!("Should not call `source` on a generated span"),
        }
    }
}


impl Diagnostic {
    fn to_ariadne(self, color: bool) -> ariadne::Report<'static, Span> {
        ariadne::Report::build(ReportKind::Error, self.label.span)
            .with_message(self.label.text)
            .with_labels(
                self.messages
                    .into_iter()
                    .map(|msg| Label::new(msg.span).with_message(msg.text)),
            )
            .with_config(Config::default().with_color(color))
            .finish()
    }
}

impl Diagnostic {
    pub fn err(span: Span, label: impl AsRef<str>, msg: impl AsRef<str>) -> Self {
        let label = Message::new(span, label.as_ref().to_string());
        let messages = vec![Message::new(span, msg.as_ref().to_string())];
        Self { label, messages }
    }

    pub fn err2(
        span0: Span,
        span1: Span,
        label: impl AsRef<str>,
        msg0: impl AsRef<str>,
        msg1: impl AsRef<str>,
    ) -> Self {
        let label = Message::new(span0, label.as_ref().to_string());
        let messages = vec![
            Message::new(span0, msg0.as_ref().to_string()),
            Message::new(span1, msg1.as_ref().to_string()),
        ];
        Self { label, messages }
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

    pub fn add(&mut self, diag: Diagnostic) {
        self.diags.push(diag);
    }

    pub fn print(&mut self, sources: &mut Cache) -> std::io::Result<()> {
        for diag in self.diags.drain(..) {
            let report = diag.to_ariadne(true);
            report.eprint(&mut *sources)?;
        }
        Ok(())
    }

    pub fn to_string(&mut self, sources: &mut Cache) -> Result<String, std::string::FromUtf8Error> {
        let mut buf = Vec::new();
        for diag in self.diags.drain(..) {
            let report = diag.to_ariadne(false);
            report.write(&mut *sources, &mut buf).unwrap();
            writeln!(&mut buf).unwrap();
        }
        String::from_utf8(buf)
    }
}
