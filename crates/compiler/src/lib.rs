use config::Config;
use diag::Report;
use diag::Sources;
use util::trim;

pub mod annotate;
pub mod apply;
pub mod ast;
pub mod codegen;
pub mod diag;
pub mod display;
// pub mod ffi;
pub mod infer;
pub mod lexer;
// pub mod lift;
pub mod opt;
pub mod parser2;
pub mod prelude;
pub mod print;
pub mod resolve;
pub mod util;

#[cfg(feature = "runtime")]
pub mod builtins;

#[cfg(feature = "runtime")]
pub mod interpret;

#[derive(Default, Debug)]
pub struct Compiler {
    _ctx0: resolve::Context,
    _ctx1: infer::Context,
    // ctx2: interpret::Context,
    pub sources: Sources,
    report: Report,
    pub config: Config,
}

impl Drop for Compiler {
    fn drop(&mut self) {
        if !self.report.is_empty() {
            self.report.print(&mut self.sources).unwrap();
        }
    }
}

impl Compiler {
    pub fn new(config: Config) -> Self {
        Self {
            _ctx0: resolve::Context::new(),
            _ctx1: infer::Context::new(),
            // ctx2: interpret::Context::new(),
            sources: Sources::new(),
            report: Report::new(),
            config,
        }
    }

    // pub fn parse<T>(
    //     &mut self,
    //     name: impl ToString,
    //     input: impl Into<Rc<str>>,
    //     f: impl for<'a> Fn(&mut Parser<'a, &mut Lexer<'a>>) -> T,
    // ) -> Result<T, (T, String)> {
    //     let input = input.into();
    //     let id = self.sources.add(name, input.clone());
    //     let mut lexer = Lexer::new(id, input.as_ref());
    //     let mut parser = Parser::new(&input, &mut lexer);
    //     let result = f(&mut parser);
    //     self.report.merge(&mut parser.report);
    //     self.report.merge(&mut lexer.report);
    //     if self.report.is_empty() {
    //         Ok(result)
    //     } else {
    //         Err((result, self.report()))
    //     }
    // }

    // pub fn resolve(&mut self, name: &str, input: &str) -> Result<Program, (Program, String)> {
    //     let program = match self.parse(name, input, |parser| parser.parse()) {
    //         Ok(program) => program,
    //         Err((program, e)) => return Err((program, e)),
    //     };
    //     let result = self.ctx0.resolve(&program);
    //     self.report.merge(&mut self.ctx0.report);
    //     if self.report.is_empty() {
    //         Ok(result)
    //     } else {
    //         Err((result, self.report()))
    //     }
    // }
    //
    // pub fn infer(&mut self, name: &str, input: &str) -> Result<Program, (Program, String)> {
    //     let program = match self.resolve(name, input) {
    //         Ok(program) => program,
    //         Err((program, e)) => return Err((program, e)),
    //     };
    //     let program = self.ctx1.infer(&program);
    //     self.report.merge(&mut self.ctx1.report);
    //     if self.report.is_empty() {
    //         Ok(program)
    //     } else {
    //         Err((program, self.report()))
    //     }
    // }

    // pub fn interpret(&mut self, name: &str, input: &str) -> Result<Program, (Program, String)> {
    //     let program = match self.infer(name, input) {
    //         Ok(program) => program,
    //         Err((program, e)) => return Err((program, e)),
    //     };
    //     self.ctx2.interpret(&program);
    //     self.report.merge(&mut self.ctx2.report);
    //     self.ok_or_err(program)
    // }

    // fn ok_or_err<T>(&mut self, result: T) -> Result<T, (T, String)> {
    //     if self.report.is_empty() {
    //         Ok(result)
    //     } else {
    //         Err((result, self.report()))
    //     }
    // }

    pub fn report(&mut self) -> String {
        trim(self.report.string(&mut self.sources).unwrap())
    }
}

// impl Expr {
//     pub fn parse(input: &str) -> Expr {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<Expr, (Option<Expr>, String)> {
//         Compiler::default()
//             .parse("test", input, |parser| parser.expr())
//             .map(|e| e.unwrap())
//     }
// }

// impl Type {
//     pub fn parse(input: &str) -> Type {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<Type, (Option<Type>, String)> {
//         Compiler::default()
//             .parse("test", input, |parser| parser.ty())
//             .map(|t| t.unwrap())
//     }
// }
//
// impl Stmt {
//     pub fn parse(input: &str) -> Stmt {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<Stmt, (Option<Stmt>, String)> {
//         Compiler::default()
//             .parse("test", input, |parser| parser.stmt())
//             .map(|s| s.unwrap())
//     }
// }
//
// impl Pat {
//     pub fn parse(input: &str) -> Pat {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<Pat, (Option<Pat>, String)> {
//         Compiler::default()
//             .parse("test", input, |parser| parser.pat())
//             .map(|p| p.unwrap())
//     }
// }
//
// impl Bound {
//     pub fn parse(input: &str) -> Bound {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<Bound, (Option<Bound>, String)> {
//         Compiler::default()
//             .parse("test", input, |parser| parser.bound())
//             .map(|t| t.unwrap())
//     }
// }
//
// impl StmtImpl {
//     pub fn parse(input: &str) -> StmtImpl {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<StmtImpl, (Option<StmtImpl>, String)> {
//         Compiler::default()
//             .parse("test", input, |parser| parser.stmt_impl())
//             .map(|i| i.unwrap())
//     }
// }
//
// impl Program {
//     pub fn parse(input: &str) -> Program {
//         Self::try_parse(input).unwrap_or_else(|(_, s)| panic!("{}", s))
//     }
//
//     pub fn diagnose(s: &str) -> String {
//         Self::try_parse(s).unwrap_err().1
//     }
//
//     pub fn try_parse(input: &str) -> Result<Program, (Program, String)> {
//         Compiler::default().parse("test", input, |parser| parser.parse())
//     }
//
//     pub fn try_resolve(input: &str) -> Result<Program, (Program, String)> {
//         Compiler::default().resolve("test", input)
//     }
//
//     pub fn try_infer(input: &str) -> Result<Program, (Program, String)> {
//         Compiler::default().infer("test", input)
//     }
// }
