use std::rc::Rc;

use crate::ast::Name;
use runtime::prelude::Aggregator;
use runtime::prelude::Assigner;
use runtime::prelude::Encoding;
use runtime::prelude::Reader;
use runtime::prelude::TimeSource;
use runtime::prelude::Writer;

use super::function::Fun;

#[derive(Debug, Clone)]
pub enum Stream {
    Source(Reader, Encoding, TimeSource<Fun>),
    Map(Rc<Stream>, Fun),
    Filter(Rc<Stream>, Fun),
    Flatten(Rc<Stream>),
    FlatMap(Rc<Stream>, Fun),
    Scan(Rc<Stream>, Fun),
    Keyby(Rc<Stream>, Fun),
    Unkey(Rc<Stream>),
    Apply(Rc<Stream>, Fun),
    Window(Rc<Stream>, Assigner, Aggregator<Fun, Fun, Fun, Fun>),
    Merge(Rc<Stream>, Rc<Stream>, Vec<Name>),
    Sink(Rc<Stream>, Writer, Encoding),
}
