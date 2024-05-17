use std::rc::Rc;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Stmt;
use crate::lexer::Lexer;
use crate::lexer::Span;
use crate::lexer::Spanned;
use crate::lexer::Token;
use crate::parser::Parser;
use crate::Compiler;

pub(crate) mod aggregator;
pub(crate) mod array;
pub(crate) mod blob;
pub(crate) mod bool;
pub(crate) mod char;
pub(crate) mod dataflow;
pub(crate) mod dict;
pub(crate) mod discretizer;
pub(crate) mod duration;
pub(crate) mod encoding;
pub(crate) mod f32;
pub(crate) mod f64;
pub(crate) mod file;
pub(crate) mod function;
pub(crate) mod i128;
pub(crate) mod i16;
pub(crate) mod i32;
pub(crate) mod i64;
pub(crate) mod i8;
pub(crate) mod image;
pub(crate) mod instance;
pub(crate) mod iterator;
pub(crate) mod keyed_stream;
pub(crate) mod matrix;
pub(crate) mod model;
pub(crate) mod never;
pub(crate) mod option;
pub(crate) mod path;
pub(crate) mod reader;
pub(crate) mod record;
pub(crate) mod result;
pub(crate) mod set;
pub(crate) mod socket;
pub(crate) mod stream;
pub(crate) mod string;
pub(crate) mod time;
pub(crate) mod time_source;
pub(crate) mod traits;
pub(crate) mod tuple;
pub(crate) mod u128;
pub(crate) mod u16;
pub(crate) mod u32;
pub(crate) mod u64;
pub(crate) mod u8;
pub(crate) mod unit;
pub(crate) mod url;
pub(crate) mod usize;
pub(crate) mod variant;
pub(crate) mod vec;
pub(crate) mod writer;

#[rustfmt::skip]
macro_rules! builtin_file_name {
    () => {{
        #[cfg(not(debug_assertions))] { "builtin" }
        #[cfg(debug_assertions)] { format!("{}:", std::panic::Location::caller()) }
    }};
}

impl Compiler {
    pub(crate) fn declare(&mut self) {
        // self.declare_aggregator();
        // self.declare_array();
        // self.declare_blob();
        // self.declare_bool();
        // self.declare_char();
        // self.declare_dict();
        // self.declare_dataflow();
        // self.declare_discretizer();
        // self.declare_duration();
        // self.declare_encoding();
        self.declare_f32();
        self.declare_f64();
        // self.declare_file();
        // self.declare_function();
        // self.declare_i128();
        // self.declare_i16();
        self.declare_i32();
        // self.declare_i64();
        // self.declare_i8();
        // self.declare_image();
        // self.declare_instance();
        // self.declare_iterator();
        // self.declare_keyed_stream();
        // self.declare_matrix();
        // self.declare_model();
        // self.declare_never();
        // self.declare_option();
        // self.declare_path();
        // self.declare_reader();
        // self.declare_record();
        // self.declare_result();
        // self.declare_set();
        // self.declare_socket();
        // self.declare_stream();
        // self.declare_string();
        // self.declare_time();
        // self.declare_time_source();
        self.declare_traits();
        // self.declare_tuple();
        // self.declare_u128();
        // self.declare_u16();
        // self.declare_u32();
        // self.declare_u64();
        // self.declare_u8();
        // self.declare_unit();
        // self.declare_url();
        // self.declare_usize();
        // self.declare_variant();
        // self.declare_vec();
        // self.declare_writer();
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_def(&mut self, source: &'static str, body: BuiltinDef) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_def_builtin(follow, body)
        }) {
            self.declarations.push(Stmt::Def(Rc::new(stmt)))
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_type(&mut self, source: &'static str, body: BuiltinType) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_type_builtin(follow, body)
        }) {
            self.declarations.push(Stmt::Type(Rc::new(stmt)))
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_trait(&mut self, source: &'static str) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_trait(follow)
        }) {
            self.declarations.push(Stmt::Trait(Rc::new(stmt)))
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn declare_impl(&mut self, source: &'static str) {
        if let Some(stmt) = self.try_parse(builtin_file_name!(), source, |parser, follow| {
            parser.stmt_impl(follow)
        }) {
            self.declarations.push(Stmt::Impl(Rc::new(stmt)))
        }
    }

    fn try_parse<T>(
        &mut self,
        name: impl ToString,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let input: Rc<str> = unindent::unindent(input).into();
        let id = self.sources.add(name, input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = parser.parse(f);
        self.report.merge(&mut parser.report);
        self.report.merge(&mut lexer.report);
        result
    }

    // fn decl_impl(&mut self, source: &'static str, body: StmtImplBody) {
    //    let lexer = crate::lexer::Lexer::new(0, source);
    //    let mut parser = crate::parser::Parser::new(source, lexer);
    //    let stmt = parser.parse(move |p, f| p.stmt_impl_builtin(f, body));
    //    self.stmts.push(stmt);
    // }
}
