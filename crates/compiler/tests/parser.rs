use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Stmt;
use compiler::ast::StmtImpl;
use compiler::ast::Type;
use compiler::util::expr_add;
use compiler::util::expr_and;
use compiler::util::expr_array;
use compiler::util::expr_assign;
use compiler::util::expr_block;
use compiler::util::expr_bool;
use compiler::util::expr_break;
use compiler::util::expr_call;
use compiler::util::expr_continue;
use compiler::util::expr_div;
use compiler::util::expr_eq;
use compiler::util::expr_err;
use compiler::util::expr_field;
use compiler::util::expr_float;
use compiler::util::expr_fun;
use compiler::util::expr_fun_typed;
use compiler::util::expr_ge;
use compiler::util::expr_gt;
use compiler::util::expr_index;
use compiler::util::expr_int;
use compiler::util::expr_le;
use compiler::util::expr_lt;
use compiler::util::expr_match;
use compiler::util::expr_mul;
use compiler::util::expr_ne;
use compiler::util::expr_neg;
use compiler::util::expr_not;
use compiler::util::expr_or;
use compiler::util::expr_query;
use compiler::util::expr_return;
use compiler::util::expr_sub;
use compiler::util::expr_tuple;
use compiler::util::expr_unit;
use compiler::util::index;
use compiler::util::pat_bool;
use compiler::util::pat_int;
use compiler::util::pat_tuple;
use compiler::util::pat_wild;
use compiler::util::program;
use compiler::util::query_compute;
use compiler::util::query_from;
use compiler::util::query_group;
use compiler::util::query_into;
use compiler::util::query_over;
use compiler::util::query_select;
use compiler::util::query_where;
use compiler::util::query_with;
use compiler::util::stmt_def;
use compiler::util::stmt_enum;
use compiler::util::stmt_expr;
use compiler::util::stmt_impl;
use compiler::util::stmt_struct;
use compiler::util::stmt_type;
use compiler::util::stmt_var;
use compiler::util::ty_err;
use compiler::util::ty_fun;
use compiler::util::ty_hole;
use compiler::util::ty_tuple;
use compiler::util::unresolved::bound;
use compiler::util::unresolved::expr_assoc;
use compiler::util::unresolved::expr_call_direct;
use compiler::util::unresolved::expr_def;
use compiler::util::unresolved::expr_struct;
use compiler::util::unresolved::expr_unit_variant;
use compiler::util::unresolved::expr_var;
use compiler::util::unresolved::expr_variant;
use compiler::util::unresolved::pat_enum;
use compiler::util::unresolved::pat_struct;
use compiler::util::unresolved::pat_unit_struct;
use compiler::util::unresolved::pat_var;
use compiler::util::unresolved::ty;
use compiler::util::unresolved::ty_assoc;
use compiler::util::unresolved::ty_con;

#[test]
fn test_parser_expr_int0() {
    let a = Expr::parse("1");
    let b = expr_int("1");
    assert_eq!(a, b);
}

#[ignore]
#[test]
fn test_parser_expr_int1() {
    let a = Expr::parse("123s");
    let b = expr_call_direct("postfix_s", [], [expr_int("123")]);
    assert_eq!(a, b);
}

#[test]
fn test_parser_expr_float0() {
    let a = Expr::parse("1.0");
    let b = expr_float("1.0");
    assert_eq!(a, b);
}

#[ignore]
#[test]
fn test_parser_expr_float1() {
    let a = Expr::parse("1.0s");
    let b = expr_call_direct("postfix_s", [], [expr_float("1.0")]);
    assert_eq!(a, b);
}

#[test]
fn test_parser_expr_binop_add0() {
    let a = Expr::parse("1 + 2");
    let b = expr_add(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_sub0() {
    let a = Expr::parse("1 - 2");
    let b = expr_sub(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_mul0() {
    let a = Expr::parse("1 * 2");
    let b = expr_mul(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_div0() {
    let a = Expr::parse("1 / 2");
    let b = expr_div(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_eq0() {
    let a = Expr::parse("1 == 2");
    let b = expr_eq(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_ne0() {
    let a = Expr::parse("1 != 2");
    let b = expr_ne(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_le0() {
    let a = Expr::parse("1 <= 2");
    let b = expr_le(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_ge0() {
    let a = Expr::parse("1 >= 2");
    let b = expr_ge(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_lt0() {
    let a = Expr::parse("1 < 2");
    let b = expr_lt(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_gt0() {
    let a = Expr::parse("1 > 2");
    let b = expr_gt(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_and0() {
    let a = Expr::parse("1 and 2");
    let b = expr_and(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_or0() {
    let a = Expr::parse("1 or 2");
    let b = expr_or(expr_int("1"), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_add_mul0() {
    let a = Expr::parse("1 + 2 * 3");
    let b = expr_add(expr_int("1"), expr_mul(expr_int("2"), expr_int("3")));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_mul_add0() {
    let a = Expr::parse("1 * 2 + 3");
    let b = expr_add(expr_mul(expr_int("1"), expr_int("2")), expr_int("3"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_add_div0() {
    let a = Expr::parse("1 + 2 / 3");
    let b = expr_add(expr_int("1"), expr_div(expr_int("2"), expr_int("3")));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_div_add0() {
    let a = Expr::parse("1 / 2 + 3");
    let b = expr_add(expr_div(expr_int("1"), expr_int("2")), expr_int("3"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_mul_div0() {
    let a = Expr::parse("1 * 2 / 3");
    let b = expr_div(expr_mul(expr_int("1"), expr_int("2")), expr_int("3"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_add_add0() {
    let a = Expr::parse("1 + 2 + 3");
    let b = expr_add(expr_add(expr_int("1"), expr_int("2")), expr_int("3"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_eq_eq0() {
    let a = Expr::parse("1 == 2 == 3");
    let b = expr_block(
        [
            stmt_var("_0", ty_hole(), expr_int("1")),
            stmt_var("_1", ty_hole(), expr_int("2")),
        ],
        expr_and(
            expr_eq(expr_var("_0"), expr_var("_1")),
            expr_eq(expr_var("_1"), expr_int("3")),
        ),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_eq_eq_eq0() {
    let a = Expr::parse("1 == 2 == 3 == 4");
    let b = expr_block(
        [
            stmt_var("_0", ty_hole(), expr_int("1")),
            stmt_var("_1", ty_hole(), expr_int("2")),
        ],
        expr_and(
            expr_eq(expr_var("_0"), expr_var("_1")),
            expr_block(
                [stmt_var("_2", ty_hole(), expr_int("3"))],
                expr_and(
                    expr_eq(expr_var("_1"), expr_var("_2")),
                    expr_eq(expr_var("_2"), expr_int("4")),
                ),
            ),
        ),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_binop_eq_eq_le0() {
    let a = Expr::parse("1 == 2 == 3 <= 4");
    let b = expr_block(
        [
            stmt_var("_0", ty_hole(), expr_int("1")),
            stmt_var("_1", ty_hole(), expr_int("2")),
        ],
        expr_and(
            expr_eq(expr_var("_0"), expr_var("_1")),
            expr_block(
                [stmt_var("_2", ty_hole(), expr_int("3"))],
                expr_and(
                    expr_eq(expr_var("_1"), expr_var("_2")),
                    expr_le(expr_var("_2"), expr_int("4")),
                ),
            ),
        ),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_unop_neg0() {
    let a = Expr::parse("-1");
    let b = expr_neg(expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_unop_neg_add0() {
    let a = Expr::parse("-1 + 2");
    let b = expr_add(expr_neg(expr_int("1")), expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_unop_not0() {
    let a = Expr::parse("!1");
    let b = expr_not(expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_call0() {
    let a = Expr::parse("x(1)");
    let b = expr_call_direct("x", [], [expr_int("1")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_call1() {
    let a = Expr::parse("1(x)");
    let b = expr_call(expr_int("1"), [expr_var("x")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_var0() {
    let a = Expr::parse("x");
    let b = expr_var("x");
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_var1() {
    let a = Expr::parse("x[i32]");
    let b = expr_def("x", [ty("i32")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_var0() {
    let a = Pat::parse("x");
    let b = pat_var("x");
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_enum0() {
    let a = Pat::parse("S::V(x)");
    let b = pat_enum("S", [], "V", pat_var("x"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_enum1() {
    let a = Pat::parse("S[i32]::V(x)");
    let b = pat_enum("S", [ty("i32")], "V", pat_var("x"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_struct0() {
    let a = Pat::parse("S(x=x,y=y)");
    let b = pat_struct("S", [], [("x", pat_var("x")), ("y", pat_var("y"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_struct1() {
    let a = Pat::parse("S[i32](x=x,y=y)");
    let b = pat_struct("S", [ty("i32")], [("x", pat_var("x")), ("y", pat_var("y"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_struct2() {
    let a = Pat::parse("S[i32]");
    let b = pat_unit_struct("S", [ty("i32")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_struct3() {
    let a = Pat::parse("S[]");
    let b = pat_unit_struct("S", []);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_if0() {
    let a = Expr::parse("if true { 1 }");
    let b = expr_match(
        expr_bool(true),
        [(pat_bool(true), expr_int("1")), (pat_wild(), expr_unit())],
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_if1() {
    let a = Program::parse("if true { 1 } if false { 2 }");
    let b = program([
        stmt_expr(expr_match(
            expr_bool(true),
            [(pat_bool(true), expr_int("1")), (pat_wild(), expr_unit())],
        )),
        stmt_expr(expr_match(
            expr_bool(false),
            [(pat_bool(true), expr_int("2")), (pat_wild(), expr_unit())],
        )),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_if_else0() {
    let a = Expr::parse("if true { 1 } else { 2 }");
    let b = expr_match(
        expr_bool(true),
        [(pat_bool(true), expr_int("1")), (pat_wild(), expr_int("2"))],
    );
    assert!(a == b, "{a}\n{b}");
}

// #[test]
// fn test_parser_expr_if_else1() {
//     let a = Expr::parse("if true { 1; 2 } else { 3; 4 }");
//     let b = expr_if_else(
//         expr_bool(true),
//         [
//             (
//                 pat_bool(true),
//                 expr_block([stmt_expr(expr_int("1"))], expr_int("2")),
//             ),
//             (
//                 pat_wild(),
//                 expr_block([stmt_expr(expr_int("3"))], expr_int("4")),
//             ),
//         ],
//     );
//     assert!(a == b, "{a}\n{b}");
// }

#[test]
fn test_parser_expr_match0() {
    let a = Expr::parse("match 1 { 1 => 2, _ => 3 }");
    let b = expr_match(
        expr_int("1"),
        [(pat_int("1"), expr_int("2")), (pat_wild(), expr_int("3"))],
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_match1() {
    let a = Expr::parse("match x { 1 => 2, _ => 3 }");
    let b = expr_match(
        expr_var("x"),
        [(pat_int("1"), expr_int("2")), (pat_wild(), expr_int("3"))],
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_method_call0() {
    let a = Expr::parse("1.foo()");
    let b = expr_call_direct("foo", [], [expr_int("1")]);
    assert!(a == b, "{a}\n{b}");
    let a = Expr::parse("1.foo(2)");
    let b = expr_call_direct("foo", [], [expr_int("1"), expr_int("2")]);
    assert!(a == b, "{a}\n{b}");
    let a = Expr::parse("1.foo(2,)");
    let b = expr_call_direct("foo", [], [expr_int("1"), expr_int("2")]);
    assert!(a == b, "{a}\n{b}");
    let a = Expr::parse("1.foo(2, 3)");
    let b = expr_call_direct("foo", [], [expr_int("1"), expr_int("2"), expr_int("3")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_method_call1() {
    let a = Expr::parse("1.foo[i32]()");
    let b = expr_call_direct("foo", [ty("i32")], [expr_int("1")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def0() {
    let a = Stmt::parse("def id(x: i32): i32 = x;");
    let b = stmt_def("id", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def1() {
    let a = Stmt::parse("def id(x: i32): i32 { x }");
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_block([], expr_var("x")),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def2() {
    let a = Stmt::parse("def id(x: i32, y: i32): i32 = x;");
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_var("x"),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def3() {
    let a = Stmt::parse("def id(x: i32, y: i32): i32 = x + y;");
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_add(expr_var("x"), expr_var("y")),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def4() {
    let a = Stmt::parse("def id(x: i32, y: i32): i32 = x + y * 2;");
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_add(expr_var("x"), expr_mul(expr_var("y"), expr_int("2"))),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def5() {
    let a = Stmt::parse("def debug(x: i32): i32 { print(x); x }");
    let b = stmt_def(
        "debug",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_block(
            [stmt_expr(expr_call_direct("print", [], [expr_var("x")]))],
            expr_var("x"),
        ),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def6() {
    let a = Stmt::parse("def f(x: i32): i32 = x;");
    let b = stmt_def("f", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x"));
    assert_eq!(a, b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def7() {
    let a = Stmt::parse("def f(x: i32,): i32 = x;");
    let b = stmt_def("f", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def8() {
    let a = Stmt::parse("def f(x: i32, y: i32): i32 = x;");
    let b = stmt_def(
        "f",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_var("x"),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_generics0() {
    let a = Stmt::parse("def f[](): i32 = 1;");
    let b = stmt_def("f", [], [], ty("i32"), [], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_generics1() {
    let a = Stmt::parse("def f[T](): i32 = 1;");
    let b = stmt_def("f", ["T"], [], ty("i32"), [], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_generics2() {
    let a = Stmt::parse("def f[T,](): i32 = 1;");
    let b = stmt_def("f", ["T"], [], ty("i32"), [], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_generics3() {
    let a = Stmt::parse("def f[T, U](): i32 = 1;");
    let b = stmt_def("f", ["T", "U"], [], ty("i32"), [], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_where0() {
    let a = Stmt::parse("def x(): i32 where = 1;");
    let b = stmt_def("x", [], [], ty("i32"), [], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_where1() {
    let a = Stmt::parse("def x(): i32 where Clone[i32] = 1;");
    let b = stmt_def(
        "x",
        [],
        [],
        ty("i32"),
        [bound("Clone", [ty("i32")])],
        expr_int("1"),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_where2() {
    let a = Stmt::parse("def x(): i32 where Clone[i32], Copy[i32] = 1;");
    let b = stmt_def(
        "x",
        [],
        [],
        ty("i32"),
        [bound("Clone", [ty("i32")]), bound("Copy", [ty("i32")])],
        expr_int("1"),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_where3() {
    let a = Stmt::parse("def x(): i32 where Clone[i32], Copy[i32], = 1;");
    let b = stmt_def(
        "x",
        [],
        [],
        ty("i32"),
        [bound("Clone", [ty("i32")]), bound("Copy", [ty("i32")])],
        expr_int("1"),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_def_where4() {
    let a = Stmt::parse("def x(): i32 where { 1 }");
    let b = stmt_def("x", [], [], ty("i32"), [], expr_block([], expr_int("1")));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_impl_where0() {
    let a = Stmt::parse("impl Copy[i32] where Clone[i32] {}");
    let b = stmt_impl(
        [],
        bound("Copy", [ty("i32")]),
        [bound("Clone", [ty("i32")])],
        [],
        [],
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program0() {
    let a = Program::parse(
        "def id(x: i32): i32 = x;
         def main(): i32 = id(42);",
    );
    let b = program([
        stmt_def("id", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x")),
        stmt_def(
            "main",
            [],
            [],
            ty("i32"),
            [],
            expr_call_direct("id", [], [expr_int("42")]),
        ),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_impl0() {
    let a = Stmt::parse(
        "impl Eq[bool] {
             def eq(x: bool, y: bool): bool = true;
         }",
    );
    let b = stmt_impl(
        [],
        bound("Eq", [ty("bool")]),
        [],
        [stmt_def(
            "eq",
            [],
            [("x", ty("bool")), ("y", ty("bool"))],
            ty("bool"),
            [],
            expr_bool(true),
        )],
        [],
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_impl1() {
    StmtImpl::parse(
        "impl[T, R] Add[Vec[T], R] where Add[T, R] {
             type Output = Vec[Add[T, R]::Output];
         }",
    );
}

#[test]
fn test_parser_stmt_var0() {
    let a = Stmt::parse("var x = 1;");
    let b = stmt_var("x", ty_hole(), expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_var1() {
    let a = Stmt::parse("var x: i32 = 1;");
    let b = stmt_var("x", ty("i32"), expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_var2() {
    let a = Program::parse("var x = 1; var y = x;");
    let b = program([
        stmt_var("x", ty_hole(), expr_int("1")),
        stmt_var("y", ty_hole(), expr_var("x")),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_assign0() {
    let a = Expr::parse("x = 1");
    let b = expr_assign(expr_var("x"), expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_type0() {
    let a = Stmt::parse("type T = i32;");
    let b = stmt_type("T", [], ty("i32"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_type1() {
    let a = Stmt::parse("type T[U] = U;");
    let b = stmt_type("T", ["U"], ty("U"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_type2() {
    let a = Stmt::parse("type T[U] = (U, U);");
    let b = stmt_type("T", ["U"], ty_tuple([ty("U"), ty("U")]));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_struct0() {
    let a = Stmt::parse("struct S;");
    let b = stmt_struct("S", [], []);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_struct_err0() {
    let a = Stmt::diagnose("struct S");
    assert_eq!(
        a,
        indoc::indoc! {"
        Error: Unexpected end of file
           ╭─[test:1:9]
           │
         1 │ struct S
           │         │
           │         ╰─ Expected `;`
        ───╯"}
    );
}

#[test]
fn test_parser_stmt_struct1() {
    let a = Stmt::parse("struct S();");
    let b = stmt_struct("S", [], []);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_struct2() {
    let a = Stmt::parse("struct S(x:i32);");
    let b = stmt_struct("S", [], [("x", ty("i32"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_struct3() {
    let a = Stmt::parse("struct S(x:i32);");
    let b = stmt_struct("S", [], [("x", ty("i32"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_struct4() {
    let a = Stmt::parse("struct S[T](x:T);");
    let b = stmt_struct("S", ["T"], [("x", ty("T"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_struct0() {
    let a = Expr::parse("S(x=1)");
    let b = expr_struct("S", [], [("x", expr_int("1"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_struct1() {
    let a = Expr::parse("S(x=x)");
    let b = expr_struct("S", [], [("x", expr_var("x"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_struct2() {
    let a = Expr::parse("s.x.y");
    let b = expr_field(expr_field(expr_var("s"), "x"), "y");
    assert!(a == b, "{a}\n{b}");
}

// Field punning is done at resolution time.
#[test]
fn test_parser_expr_struct3() {
    let a = Expr::parse("S(x=s.x)");
    let b = expr_struct("S", [], [("x", expr_field(expr_var("s"), "x"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_struct5() {
    let a = Expr::parse("S[i32](x=1)");
    let b = expr_struct("S", [ty("i32")], [("x", expr_int("1"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_enum0() {
    let a = Stmt::parse("enum E { A(i32), B(i32) }");
    let b = stmt_enum("E", [], [("A", ty("i32")), ("B", ty("i32"))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_enum0() {
    let a = Expr::parse("E::A");
    let b = expr_unit_variant("E", [], "A");
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_enum1() {
    let a = Expr::parse("E::A()");
    let b = expr_variant("E", [], "A", []);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_enum2() {
    let a = Expr::parse("E::A(1,)");
    let b = expr_variant("E", [], "A", [expr_int("1")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_enum3() {
    let a = Expr::parse("E::A(1, 2)");
    let b = expr_variant("E", [], "A", [expr_int("1"), expr_int("2")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_enum4() {
    let a = Expr::parse("E[i32]::A(1, 2)");
    let b = expr_variant("E", [ty("i32")], "A", [expr_int("1"), expr_int("2")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_array0() {
    let a = Expr::parse("[1, 2, 3]");
    let b = expr_array([expr_int("1"), expr_int("2"), expr_int("3")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_tuple0() {
    let a = Expr::parse("()");
    let b = expr_tuple([]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_tuple1() {
    let a = Expr::parse("(1,)");
    let b = expr_tuple([expr_int("1")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_tuple2() {
    let a = Expr::parse("(1, 2)");
    let b = expr_tuple([expr_int("1"), expr_int("2")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_tuple3() {
    let a = Expr::parse("a.0");
    let b = expr_index(expr_var("a"), index("0"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_tuple0() {
    let a = Pat::parse("()");
    let b = pat_tuple([]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_tuple1() {
    let a = Pat::parse("(1,)");
    let b = pat_tuple([pat_int("1")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_pat_tuple2() {
    let a = Pat::parse("(1, 2)");
    let b = pat_tuple([pat_int("1"), pat_int("2")]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace0() {
    let a = Program::parse("{1}");
    let b = program([stmt_expr(expr_block([], expr_int("1")))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace1() {
    let a = Program::parse("{1} {2}");
    let b = program([
        stmt_expr(expr_block([], expr_int("1"))),
        stmt_expr(expr_block([], expr_int("2"))),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace2() {
    let a = Program::parse("{{1}}");
    let b = program([stmt_expr(expr_block([], expr_block([], expr_int("1"))))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace3() {
    let a = Program::parse("{{1} {2}}");
    let b = program([stmt_expr(expr_block(
        [stmt_expr(expr_block([], expr_int("1")))],
        expr_block([], expr_int("2")),
    ))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace4() {
    let a = Program::parse("{{1};{2}}");
    let b = program([stmt_expr(expr_block(
        [stmt_expr(expr_block([], expr_int("1")))],
        expr_block([], expr_int("2")),
    ))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace5() {
    let a = Program::parse("{{1};{2};}");
    let b = program([stmt_expr(expr_block(
        [
            stmt_expr(expr_block([], expr_int("1"))),
            stmt_expr(expr_block([], expr_int("2"))),
        ],
        expr_unit(),
    ))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace6() {
    let a = Program::parse("{;}");
    let b = program([stmt_expr(expr_block([], expr_unit()))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace7() {
    let a = Program::parse("{;;;;;;;;}");
    let b = program([stmt_expr(expr_block([], expr_unit()))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_brace8() {
    let a = Expr::parse("{1;2}");
    let b = expr_block([stmt_expr(expr_int("1"))], expr_int("2"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_paren0() {
    let a = Program::parse("();");
    let b = program([stmt_expr(expr_unit())]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_paren1() {
    let a = Program::parse("(());");
    let b = program([stmt_expr(expr_unit())]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_program_paren2() {
    let a = Program::parse("({});");
    let b = program([stmt_expr(expr_block([], expr_unit()))]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_assoc0() {
    let a = Expr::parse("Iterator[Vec[i32]]::next");
    let b = expr_assoc("Iterator", [ty_con("Vec", [ty("i32")])], "next");
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_type_assoc0() {
    let a = Type::parse("Iterator[Vec[i32]]::Item");
    let b = ty_assoc("Iterator", [ty_con("Vec", [ty("i32")])], "Item");
    assert_eq!(a, b);
}

#[test]
fn test_parser_expr_query0() {
    let a = Expr::parse("from x in [1, 2, 3] ");
    let b = expr_query([query_from([(
        "x",
        expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
    )])]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query1() {
    let a = Expr::parse(
        "from x in source()
         select {x=f(), y=g()}
         into sink()",
    );
    let b = expr_query([
        query_from([("x", expr_call_direct("source", [], []))]),
        query_select([
            ("x", expr_call_direct("f", [], [])),
            ("y", expr_call_direct("g", [], [])),
        ]),
        query_into([expr_call_direct("sink", [], [])]),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query2() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select {x=1, y=2}
         where x > 1",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query3() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select {x=1, y=2}
         where x > 1
         select {x=1, y=2}",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
        query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query4() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         with y = f(x)",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_with([("y", expr_call_direct("f", [], [expr_var("x")]))]),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query5() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         group x {
             select {x=1, y=2}
             with z = f(x)
             where x > 1
         }",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_group(
            ["x"],
            [
                query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
                query_with([("z", expr_call_direct("f", [], [expr_var("x")]))]),
                query_where(expr_gt(expr_var("x"), expr_int("1"))),
            ],
        ),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query6() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         group x {
             compute total = sum of x
         }",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_group(
            ["x"],
            [query_compute([("total", expr_var("sum"), expr_var("x"))])],
        ),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query7() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         group x {
             compute {
                 total = sum of x,
                 lowest = min of x,
                 highest = max of x
             }
         }",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_group(
            ["x"],
            [query_compute([
                ("total", expr_var("sum"), expr_var("x")),
                ("lowest", expr_var("min"), expr_var("x")),
                ("highest", expr_var("max"), expr_var("x")),
            ])],
        ),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query8() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         over tumbling(1) {
             compute {
                 total = sum of x,
                 lowest = min of x,
                 highest = max of x
             }
             select {x=1, y=2}
             where x > 1
         }",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_over(
            expr_call_direct("tumbling", [], [expr_int("1")]),
            [
                query_compute([
                    ("total", expr_var("sum"), expr_var("x")),
                    ("lowest", expr_var("min"), expr_var("x")),
                    ("highest", expr_var("max"), expr_var("x")),
                ]),
                query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
                query_where(expr_gt(expr_var("x"), expr_int("1"))),
            ],
        ),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query9() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select x=1, y=2, z=3
         where x > 1",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_select([
            ("x", expr_int("1")),
            ("y", expr_int("2")),
            ("z", expr_int("3")),
        ]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
    ]);
    assert!(a == b, "{a:?}\n{b:?}");
}

#[test]
fn test_parser_expr_query10() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select x=1, y=2, z=3
         where x > 1",
    );
    let b = expr_query([
        query_from([(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        )]),
        query_select([
            ("x", expr_int("1")),
            ("y", expr_int("2")),
            ("z", expr_int("3")),
        ]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
    ]);
    assert!(a == b, "{a:?}\n{b:?}");
}

#[test]
fn test_parser_expr_query11() {
    let a = Expr::parse(
        "from x in [1, 2, 3],
              y in [1, 2, 3]
         compute
           highest = max of x,
           lowest = min of x
         select x, y, z=3
         with
           x = f(x),
           y = g(x)
         where x > 1
         into sink()",
    );
    let b = expr_query([
        query_from([
            (
                "x",
                expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
            ),
            (
                "y",
                expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
            ),
        ]),
        query_compute([
            ("highest", expr_var("max"), expr_var("x")),
            ("lowest", expr_var("min"), expr_var("x")),
        ]),
        query_select([
            ("x", expr_var("x")),
            ("y", expr_var("y")),
            ("z", expr_int("3")),
        ]),
        query_with([
            ("x", expr_call_direct("f", [], [expr_var("x")])),
            ("y", expr_call_direct("g", [], [expr_var("x")])),
        ]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
        query_into([expr_call_direct("sink", [], [])]),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_query12() {
    let a = Expr::parse(
        "from x in [1, 2, 3],
              y in [1, 2, 3]
         group x, y {
             select {x=1, y=2}
         }
         into sink()",
    );
    let b = expr_query([
        query_from([
            (
                "x",
                expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
            ),
            (
                "y",
                expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
            ),
        ]),
        query_group(
            ["x", "y"],
            [query_select([("x", expr_int("1")), ("y", expr_int("2"))])],
        ),
        query_into([expr_call_direct("sink", [], [])]),
    ]);
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_stmt_query() {
    let a = Stmt::parse(
        "from x in [1, 2, 3],
              y in [1, 2, 3]
         compute
           highest = max of x,
           lowest = min of x
         select x, y, z=3
         with
           x = f(x),
           y = g(x)
         where x > 1
         into sink(),
              sink();",
    );
    let b = stmt_expr(expr_query([
        query_from([
            (
                "x",
                expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
            ),
            (
                "y",
                expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
            ),
        ]),
        query_compute([
            ("highest", expr_var("max"), expr_var("x")),
            ("lowest", expr_var("min"), expr_var("x")),
        ]),
        query_select([
            ("x", expr_var("x")),
            ("y", expr_var("y")),
            ("z", expr_int("3")),
        ]),
        query_with([
            ("x", expr_call_direct("f", [], [expr_var("x")])),
            ("y", expr_call_direct("g", [], [expr_var("x")])),
        ]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
        query_into([
            expr_call_direct("sink", [], []),
            expr_call_direct("sink", [], []),
        ]),
    ]));
    assert_eq!(a, b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_fun0() {
    let a = Expr::parse("fun() = 1");
    let b = expr_fun([], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_fun1() {
    let a = Expr::parse("fun(x: i32): i32 = 1");
    let b = expr_fun_typed([("x", ty("i32"))], ty("i32"), expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_fun2() {
    let a = Expr::parse("fun(x) = fun(y) = 1");
    let b = expr_fun(["x"], expr_fun(["y"], expr_int("1")));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_type_fun0() {
    let a = Type::parse("fun(i32, i32): i32");
    let b = ty_fun([ty("i32"), ty("i32")], ty("i32"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_type_fun1() {
    let a = Type::parse("fun(i32): fun(i32): i32");
    let b = ty_fun([ty("i32")], ty_fun([ty("i32")], ty("i32")));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_return0() {
    let a = Expr::parse("return 1");
    let b = expr_return(expr_int("1"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_return1() {
    let a = Expr::parse("return");
    let b = expr_return(expr_unit());
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_continue0() {
    let a = Expr::parse("continue");
    let b = expr_continue();
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_break0() {
    let a = Expr::parse("break");
    let b = expr_break();
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_unexpected_eof0() {
    let a = Stmt::diagnose("def f(x: i32): i32 = 1");
    assert_eq!(
        a,
        indoc::indoc! {"
            Error: Unexpected end of file
               ╭─[test:1:23]
               │
             1 │ def f(x: i32): i32 = 1
               │                       │
               │                       ╰─ Expected `;`
            ───╯"}
    );
}

#[test]
fn test_parser_unexpected_token0() {
    let a = Stmt::diagnose("def f(x: i32): i32 = 1 2");
    assert_eq!(
        a,
        indoc::indoc! {"
            Error: Unexpected token `<int>`
               ╭─[test:1:24]
               │
             1 │ def f(x: i32): i32 = 1 2
               │                        ┬
               │                        ╰── Expected `;`
            ───╯"}
    );
}

#[test]
fn test_parser_recover0() {
    let a = Stmt::parse("def f(x: i32): i32 = +;");
    let b = stmt_def(
        "f",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_add(expr_err(), expr_err()),
    );
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_recover1() {
    let (a, msg) = Stmt::try_parse("def f(x: +): i32 = 1;").unwrap_err();
    let a = a.unwrap();
    let b = stmt_def("f", [], [("x", ty_err())], ty("i32"), [], expr_int("1"));
    assert!(a == b, "{a}\n{b}");
    assert_eq!(
        msg,
        indoc::indoc! {"
            Error: Unexpected token `+`
               ╭─[test:1:10]
               │
             1 │ def f(x: +): i32 = 1;
               │          ┬
               │          ╰── Expected `)`
            ───╯"},
    )
}

#[test]
fn test_parser_pat_annotate0() {
    let a = Pat::try_parse("1:i32").unwrap();
    let b = pat_int("1").with_ty(ty("i32"));
    assert!(a == b, "{a}\n{b}");
}

#[test]
fn test_parser_expr_annotate0() {
    let a = Expr::try_parse("1:i32").unwrap();
    let b = expr_int("1").of(ty("i32"));
    assert!(a == b, "{a}\n{b}");
}

#[ignore]
#[test]
fn test_parser_depth0() {
    let r = format!("{}{}", "{".repeat(1000), "}".repeat(1000));
    let _ = Expr::parse(&r);
}
