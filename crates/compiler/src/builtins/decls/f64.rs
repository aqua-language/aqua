use crate::ast::Bound;
use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Map;
use crate::ast::Path;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtImpl;
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::TraitBound;
use crate::ast::Type;
use crate::lexer::Span;
use crate::Compiler;
use std::rc::Rc;

impl Compiler {
    pub(super) fn declare_f64(&mut self) {
        self.declare_type("type f64;", BuiltinType { rust: "f64" });

        self.declare_impl(
            "impl Add[f64,f64] {
                type Output = f64;
                def add(a:f64, b:f64): f64;
            }",
            [BuiltinDef {
                rust: "f64::add_f64",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 + v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Sub[f64,f64] {
                type Output = f64;
                def sub(a:f64, b:f64): f64;
            }",
            [BuiltinDef {
                rust: "f64::sub_f64",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 - v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Mul[f64,f64] {
                type Output = f64;
                def mul(a:f64, b:f64): f64;
            }",
            [BuiltinDef {
                rust: "f64::mul_f64",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 * v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Div[f64,f64] {
                type Output = f64;
                def div(a:f64, b:f64): f64;
            }",
            [BuiltinDef {
                rust: "f64::div_f64",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 / v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Add[f64,i32] {
                type Output = f64;
                def add(a:f64, b:i32): f64;
            }",
            [BuiltinDef {
                rust: "f64::add_i32",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_i32();
                    (v0 + v1 as f64).into()
                },
            }],
        );

        self.declare_impl(
            "impl Add[i32,f64] {
                type Output = f64;
                def add(a:i32, b:f64): f64;
            }",
            [BuiltinDef {
                rust: "f64::add_i32",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_f64();
                    (v0 as f64 + v1).into()
                },
            }],
        );
    }
}
