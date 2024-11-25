use crate::ast::Expr;
use crate::ast::Program;
use crate::ast::Type;
use crate::diag::Report;
use crate::traversal::visitor::Visitor;

struct Context {
    report: Report,
}

impl Context {
    fn new() -> Self {
        Self {
            report: Report::new(),
        }
    }
}

pub fn check(program: &Program) -> Report {
    let mut ctx = Context::new();
    ctx.visit_program(program);
    ctx.report
}

impl Visitor for Context {
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(s, t, v) => {
                let Type::Builtin(x, _) = t else { unreachable!() };
                let r = match x.data.as_str() {
                    "i8" => v.as_str().parse::<i8>().err().map(|e| e.to_string()),
                    "i16" => v.as_str().parse::<i16>().err().map(|e| e.to_string()),
                    "i32" => v.as_str().parse::<i32>().err().map(|e| e.to_string()),
                    "i64" => v.as_str().parse::<i64>().err().map(|e| e.to_string()),
                    "u8" => v.as_str().parse::<u8>().err().map(|e| e.to_string()),
                    "u16" => v.as_str().parse::<u16>().err().map(|e| e.to_string()),
                    "u32" => v.as_str().parse::<u32>().err().map(|e| e.to_string()),
                    "u64" => v.as_str().parse::<u64>().err().map(|e| e.to_string()),
                    _ => unreachable!(),
                };
                if let Some(e) = r {
                    self.report.err(*s, "Integer parsing error", e);
                }
            }
            Expr::Float(s, t, v) => {
                let Type::Builtin(x, _) = t else { unreachable!() };
                let r = match x.data.as_str() {
                    "f32" => v.as_str().parse::<f32>().err().map(|e| e.to_string()),
                    "f64" => v.as_str().parse::<f64>().err().map(|e| e.to_string()),
                    _ => unreachable!(),
                };
                if let Some(e) = r {
                    self.report.err(*s, "Float parsing error", e);
                }
            }
            _ => self._visit_expr(expr),
        }
    }
}
