use std::rc::Rc;

use runtime::builtins::encoding::Encoding;
use runtime::builtins::writer::Writer;

use crate::ast::Name;

use super::stream::Stream;

#[derive(Debug, Clone)]
pub struct Dataflow {
    pub streams: Vec<Stream>,
    pub sinks: Vec<Sink>,
}

impl Dataflow {
    pub fn new(streams: Vec<Stream>, sinks: Vec<Sink>) -> Self {
        Self { streams, sinks }
    }
}

#[derive(Debug, Clone)]
pub struct Sink(pub Rc<(Name, Writer, Encoding)>);

impl Sink {
    pub fn new(stream: Name, writer: Writer, encoding: Encoding) -> Self {
        Self(Rc::new((stream, writer, encoding)))
    }
}
