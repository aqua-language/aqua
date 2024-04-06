use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Stmt;
use compiler::ast::Type;
use compiler::check;
use compiler::dsl::block;
use compiler::dsl::expr_and;
use compiler::dsl::expr_array;
use compiler::dsl::expr_assign;
use compiler::dsl::expr_block;
use compiler::dsl::expr_bool;
use compiler::dsl::expr_break;
use compiler::dsl::expr_call;
use compiler::dsl::expr_char;
use compiler::dsl::expr_continue;
use compiler::dsl::expr_err;
use compiler::dsl::expr_field;
use compiler::dsl::expr_float;
use compiler::dsl::expr_fun;
use compiler::dsl::expr_fun_typed;
use compiler::dsl::expr_if;
use compiler::dsl::expr_if_else;
use compiler::dsl::expr_index;
use compiler::dsl::expr_int;
use compiler::dsl::expr_match;
use compiler::dsl::expr_or;
use compiler::dsl::expr_query;
use compiler::dsl::expr_return;
use compiler::dsl::expr_string;
use compiler::dsl::expr_tuple;
use compiler::dsl::expr_unit;
use compiler::dsl::expr_while;
use compiler::dsl::index;
use compiler::dsl::pat_bool;
use compiler::dsl::pat_char;
use compiler::dsl::pat_int;
use compiler::dsl::pat_record;
use compiler::dsl::pat_string;
use compiler::dsl::pat_tuple;
use compiler::dsl::pat_wild;
use compiler::dsl::program;
use compiler::dsl::query_compute;
use compiler::dsl::query_from;
use compiler::dsl::query_group;
use compiler::dsl::query_into;
use compiler::dsl::query_over;
use compiler::dsl::query_select;
use compiler::dsl::query_var;
use compiler::dsl::query_where;
use compiler::dsl::stmt_def;
use compiler::dsl::stmt_enum;
use compiler::dsl::stmt_err;
use compiler::dsl::stmt_expr;
use compiler::dsl::stmt_impl;
use compiler::dsl::stmt_struct;
use compiler::dsl::stmt_trait;
use compiler::dsl::stmt_type;
use compiler::dsl::stmt_var;
use compiler::dsl::tr_def;
use compiler::dsl::tr_type;
use compiler::dsl::ty_err;
use compiler::dsl::ty_fun;
use compiler::dsl::ty_hole;
use compiler::dsl::ty_tuple;
use compiler::dsl::unresolved::bound;
use compiler::dsl::unresolved::expr_add;
use compiler::dsl::unresolved::expr_assoc;
use compiler::dsl::unresolved::expr_call_direct;
use compiler::dsl::unresolved::expr_def;
use compiler::dsl::unresolved::expr_div;
use compiler::dsl::unresolved::expr_eq;
use compiler::dsl::unresolved::expr_ge;
use compiler::dsl::unresolved::expr_gt;
use compiler::dsl::unresolved::expr_le;
use compiler::dsl::unresolved::expr_lt;
use compiler::dsl::unresolved::expr_mul;
use compiler::dsl::unresolved::expr_ne;
use compiler::dsl::unresolved::expr_neg;
use compiler::dsl::unresolved::expr_not;
use compiler::dsl::unresolved::expr_struct;
use compiler::dsl::unresolved::expr_sub;
use compiler::dsl::unresolved::expr_unit_variant;
use compiler::dsl::unresolved::expr_var;
use compiler::dsl::unresolved::expr_variant;
use compiler::dsl::unresolved::head;
use compiler::dsl::unresolved::pat_enum;
use compiler::dsl::unresolved::pat_struct;
use compiler::dsl::unresolved::pat_unit_struct;
use compiler::dsl::unresolved::pat_var;
use compiler::dsl::unresolved::ty;
use compiler::dsl::unresolved::ty_assoc;
use compiler::dsl::unresolved::ty_con;

#[test]
fn test_parser_expr_int0() {
    let a = Expr::parse("1").unwrap();
    let b = expr_int("1");
    check!(a, b);
}

#[ignore]
#[test]
fn test_parser_expr_int1() {
    let a = Expr::parse("123s").unwrap();
    let b = expr_call_direct("postfix_s", [], [expr_int("123")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_float0() {
    let a = Expr::parse("1.0").unwrap();
    let b = expr_float("1.0");
    check!(a, b);
}

#[ignore]
#[test]
fn test_parser_expr_float1() {
    let a = Expr::parse("1.0s").unwrap();
    let b = expr_call_direct("postfix_s", [], [expr_float("1.0")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_string0() {
    let a = Expr::parse("\"foo\"").unwrap();
    let b = expr_string("foo");
    check!(a, b);
}

#[test]
fn test_parser_expr_char0() {
    let a = Expr::parse("'a'").unwrap();
    let b = expr_char('a');
    check!(a, b);
}

#[test]
fn test_parser_expr_bool0() {
    let a = Expr::parse("true").unwrap();
    let b = expr_bool(true);
    check!(a, b);
}

#[test]
fn test_parser_expr_bool1() {
    let a = Expr::parse("false").unwrap();
    let b = expr_bool(false);
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_add0() {
    let a = Expr::parse("1 + 2").unwrap();
    let b = expr_add(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_sub0() {
    let a = Expr::parse("1 - 2").unwrap();
    let b = expr_sub(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_mul0() {
    let a = Expr::parse("1 * 2").unwrap();
    let b = expr_mul(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_div0() {
    let a = Expr::parse("1 / 2").unwrap();
    let b = expr_div(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_eq0() {
    let a = Expr::parse("1 == 2").unwrap();
    let b = expr_eq(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_ne0() {
    let a = Expr::parse("1 != 2").unwrap();
    let b = expr_ne(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_le0() {
    let a = Expr::parse("1 <= 2").unwrap();
    let b = expr_le(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_ge0() {
    let a = Expr::parse("1 >= 2").unwrap();
    let b = expr_ge(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_lt0() {
    let a = Expr::parse("1 < 2").unwrap();
    let b = expr_lt(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_gt0() {
    let a = Expr::parse("1 > 2").unwrap();
    let b = expr_gt(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_and0() {
    let a = Expr::parse("1 and 2").unwrap();
    let b = expr_and(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_or0() {
    let a = Expr::parse("1 or 2").unwrap();
    let b = expr_or(expr_int("1"), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_add_mul0() {
    let a = Expr::parse("1 + 2 * 3").unwrap();
    let b = expr_add(expr_int("1"), expr_mul(expr_int("2"), expr_int("3")));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_mul_add0() {
    let a = Expr::parse("1 * 2 + 3").unwrap();
    let b = expr_add(expr_mul(expr_int("1"), expr_int("2")), expr_int("3"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_add_div0() {
    let a = Expr::parse("1 + 2 / 3").unwrap();
    let b = expr_add(expr_int("1"), expr_div(expr_int("2"), expr_int("3")));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_div_add0() {
    let a = Expr::parse("1 / 2 + 3").unwrap();
    let b = expr_add(expr_div(expr_int("1"), expr_int("2")), expr_int("3"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_mul_div0() {
    let a = Expr::parse("1 * 2 / 3").unwrap();
    let b = expr_div(expr_mul(expr_int("1"), expr_int("2")), expr_int("3"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_add_add0() {
    let a = Expr::parse("1 + 2 + 3").unwrap();
    let b = expr_add(expr_add(expr_int("1"), expr_int("2")), expr_int("3"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_eq_eq0() {
    let a = Expr::parse("1 == 2 == 3").unwrap();
    let b = expr_eq(expr_eq(expr_int("1"), expr_int("2")), expr_int("3"));
    check!(a, b);
}

#[test]
fn test_parser_expr_binop_eq_le0() {
    let a = Expr::parse("1 == 2 <= 3").unwrap();
    let b = expr_le(expr_eq(expr_int("1"), expr_int("2")), expr_int("3"));
    check!(a, b);
}

#[test]
fn test_parser_expr_unop_neg0() {
    let a = Expr::parse("-1").unwrap();
    let b = expr_neg(expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_expr_unop_neg_add0() {
    let a = Expr::parse("-1 + 2").unwrap();
    let b = expr_add(expr_neg(expr_int("1")), expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_expr_unop_not0() {
    let a = Expr::parse("!1").unwrap();
    let b = expr_not(expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_expr_call0() {
    let a = Expr::parse("x(1)").unwrap();
    let b = expr_call_direct("x", [], [expr_int("1")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_call1() {
    let a = Expr::parse("1(x)").unwrap();
    let b = expr_call(expr_int("1"), [expr_var("x")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_var0() {
    let a = Expr::parse("x").unwrap();
    let b = expr_var("x");
    check!(a, b);
}

#[test]
fn test_parser_expr_var1() {
    let a = Expr::parse("x[i32]").unwrap();
    let b = expr_def("x", [ty("i32")]);
    check!(a, b);
}

#[test]
fn test_parser_pat_int0() {
    let a = Pat::parse("1").unwrap();
    let b = pat_int("1");
    check!(a, b);
}

#[test]
fn test_parser_pat_string0() {
    let a = Pat::parse("\"foo\"").unwrap();
    let b = pat_string("foo");
    check!(a, b);
}

#[test]
fn test_parser_pat_char0() {
    let a = Pat::parse("'a'").unwrap();
    let b = pat_char('a');
    check!(a, b);
}

#[test]
fn test_parser_pat_bool0() {
    let a = Pat::parse("true").unwrap();
    let b = pat_bool(true);
    check!(a, b);
}

#[test]
fn test_parser_pat_bool1() {
    let a = Pat::parse("false").unwrap();
    let b = pat_bool(false);
    check!(a, b);
}

#[test]
fn test_parser_pat_var0() {
    let a = Pat::parse("x").unwrap();
    let b = pat_var("x");
    check!(a, b);
}

#[test]
fn test_parser_pat_enum0() {
    let a = Pat::parse("S::V(x)").unwrap();
    let b = pat_enum("S", [], "V", pat_var("x"));
    check!(a, b);
}

#[test]
fn test_parser_pat_enum1() {
    let a = Pat::parse("S[i32]::V(x)").unwrap();
    let b = pat_enum("S", [ty("i32")], "V", pat_var("x"));
    check!(a, b);
}

#[test]
fn test_parser_pat_struct0() {
    let a = Pat::parse("S(x=1,y=2)").unwrap();
    let b = pat_struct("S", [], [("x", pat_int("1")), ("y", pat_int("2"))]);
    check!(a, b);
}

#[test]
fn test_parser_pat_struct1() {
    let a = Pat::parse("S[i32](x=1,y=2)").unwrap();
    let b = pat_struct("S", [ty("i32")], [("x", pat_int("1")), ("y", pat_int("2"))]);
    check!(a, b);
}

#[test]
fn test_parser_pat_struct2() {
    let a = Pat::parse("S[i32]").unwrap();
    let b = pat_unit_struct("S", [ty("i32")]);
    check!(a, b);
}

#[test]
fn test_parser_pat_struct3() {
    let a = Pat::parse("S[]").unwrap();
    let b = pat_unit_struct("S", []);
    check!(a, b);
}

#[test]
fn test_parser_pat_inline_struct0() {
    let a = Pat::parse("struct()").unwrap();
    let b = pat_record([]);
    check!(a, b);
}

#[test]
fn test_parser_pat_inline_struct1() {
    let a = Pat::parse("struct(x=1)").unwrap();
    let b = pat_record([("x", pat_int("1"))]);
    check!(a, b);
}

#[test]
fn test_parser_pat_inline_struct2() {
    let a = Pat::parse("struct(x=1,y=2)").unwrap();
    let b = pat_record([("x", pat_int("1")), ("y", pat_int("2"))]);
    check!(a, b);
}

#[test]
fn test_parser_pat_tuple0() {
    let a = Pat::parse("()").unwrap();
    let b = pat_tuple([]);
    check!(a, b);
}

#[test]
#[ignore]
fn test_parser_pat_tuple1() {
    let a = Pat::parse("(1,)").unwrap();
    let b = pat_tuple([pat_int("1")]);
    check!(a, b);
}

#[test]
fn test_parser_pat_tuple2() {
    let a = Pat::parse("(1, 2)").unwrap();
    let b = pat_tuple([pat_int("1"), pat_int("2")]);
    check!(a, b);
}

#[test]
fn test_parser_pat_annotate0() {
    let a = Pat::parse("1:i32").unwrap();
    let b = pat_int("1").with_ty(ty("i32"));
    check!(a, b);
}

#[test]
fn test_parser_expr_annotate0() {
    let a = Expr::parse("1:i32").unwrap();
    let b = expr_int("1").with_ty(ty("i32"));
    check!(a, b);
}

#[test]
fn test_parser_expr_if0() {
    let a = Expr::parse("if true { 1 }").unwrap();
    let b = expr_if(expr_bool(true), block([], expr_int("1")));
    check!(a, b);
}

#[test]
fn test_parser_expr_if1() {
    let a = Program::parse("if true { 1 } if false { 2 }").unwrap();
    let b = program([
        stmt_expr(expr_if(expr_bool(true), block([], expr_int("1")))),
        stmt_expr(expr_if(expr_bool(false), block([], expr_int("2")))),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_if2() {
    let a = Expr::parse("if x { 1 }").unwrap();
    let b = expr_if(expr_var("x"), block([], expr_int("1")));
    check!(a, b);
}

#[test]
fn test_parser_expr_if_else0() {
    let a = Expr::parse("if true { 1 } else { 2 }").unwrap();
    let b = expr_if_else(
        expr_bool(true),
        block([], expr_int("1")),
        block([], expr_int("2")),
    );
    check!(a, b);
}

#[test]
fn test_parser_expr_if_else1() {
    let a = Expr::parse("if true { 1; 2 } else { 3; 4 }").unwrap();
    let b = expr_if_else(
        expr_bool(true),
        block([stmt_expr(expr_int("1"))], expr_int("2")),
        block([stmt_expr(expr_int("3"))], expr_int("4")),
    );
    check!(a, b);
}

#[test]
fn test_parser_expr_match0() {
    let a = Expr::parse("match 1 { 1 => 2, _ => 3 }").unwrap();
    let b = expr_match(
        expr_int("1"),
        [(pat_int("1"), expr_int("2")), (pat_wild(), expr_int("3"))],
    );
    check!(a, b);
}

#[test]
fn test_parser_expr_match1() {
    let a = Expr::parse("match x { 1 => 2, _ => 3 }").unwrap();
    let b = expr_match(
        expr_var("x"),
        [(pat_int("1"), expr_int("2")), (pat_wild(), expr_int("3"))],
    );
    check!(a, b);
}

#[test]
fn test_parser_expr_match2() {
    let a = Program::parse("match x { } match y { }").unwrap();
    let b = program([
        stmt_expr(expr_match(expr_var("x"), [])),
        stmt_expr(expr_match(expr_var("y"), [])),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_while0() {
    let a = Expr::parse("while true { 1 }").unwrap();
    let b = expr_while(expr_bool(true), block([], expr_int("1")));
    check!(a, b);
}

#[test]
fn test_parser_expr_method_call0() {
    let a = Expr::parse("1.foo()").unwrap();
    let b = expr_call_direct("foo", [], [expr_int("1")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_method_call1() {
    let a = Expr::parse("1.foo(2)").unwrap();
    let b = expr_call_direct("foo", [], [expr_int("1"), expr_int("2")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_method_call2() {
    let a = Expr::parse("1.foo(2,)").unwrap();
    let b = expr_call_direct("foo", [], [expr_int("1"), expr_int("2")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_method_call3() {
    let a = Expr::parse("1.foo(2, 3)").unwrap();
    let b = expr_call_direct("foo", [], [expr_int("1"), expr_int("2"), expr_int("3")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_method_call4() {
    let a = Expr::parse("1.foo[i32]()").unwrap();
    let b = expr_call_direct("foo", [ty("i32")], [expr_int("1")]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_def0() {
    let a = Stmt::parse("def id(x: i32): i32 = x;").unwrap();
    let b = stmt_def("id", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def1() {
    let a = Stmt::parse("def id(x: i32): i32 = { x }").unwrap();
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_block([], expr_var("x")),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def2() {
    let a = Stmt::parse("def id(x: i32, y: i32): i32 = x;").unwrap();
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_var("x"),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def3() {
    let a = Stmt::parse("def id(x: i32, y: i32): i32 = x + y;").unwrap();
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_add(expr_var("x"), expr_var("y")),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def4() {
    let a = Stmt::parse("def id(x: i32, y: i32): i32 = x + y * 2;").unwrap();
    let b = stmt_def(
        "id",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_add(expr_var("x"), expr_mul(expr_var("y"), expr_int("2"))),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def5() {
    let a = Stmt::parse("def debug(x: i32): i32 = { print(x); x }").unwrap();
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
    check!(a, b);
}

#[test]
fn test_parser_stmt_def6() {
    let a = Stmt::parse("def f(x: i32): i32 = x;").unwrap();
    let b = stmt_def("f", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def7() {
    let a = Stmt::parse("def f(x: i32,): i32 = x;").unwrap();
    let b = stmt_def("f", [], [("x", ty("i32"))], ty("i32"), [], expr_var("x"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def8() {
    let a = Stmt::parse("def f(x: i32, y: i32): i32 = x;").unwrap();
    let b = stmt_def(
        "f",
        [],
        [("x", ty("i32")), ("y", ty("i32"))],
        ty("i32"),
        [],
        expr_var("x"),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_generics0() {
    let a = Stmt::parse("def f[](): i32 = 1;").unwrap();
    let b = stmt_def("f", [], [], ty("i32"), [], expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_generics1() {
    let a = Stmt::parse("def f[T](): i32 = 1;").unwrap();
    let b = stmt_def("f", ["T"], [], ty("i32"), [], expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_generics2() {
    let a = Stmt::parse("def f[T,](): i32 = 1;").unwrap();
    let b = stmt_def("f", ["T"], [], ty("i32"), [], expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_generics3() {
    let a = Stmt::parse("def f[T, U](): i32 = 1;").unwrap();
    let b = stmt_def("f", ["T", "U"], [], ty("i32"), [], expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_where0() {
    let a = Stmt::parse("def x(): i32 where = 1;").unwrap();
    let b = stmt_def("x", [], [], ty("i32"), [], expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_where1() {
    let a = Stmt::parse("def x(): i32 where Clone[i32] = 1;").unwrap();
    let b = stmt_def(
        "x",
        [],
        [],
        ty("i32"),
        [bound("Clone", [ty("i32")], [])],
        expr_int("1"),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_where2() {
    let a = Stmt::parse("def x(): i32 where Clone[i32], Copy[i32] = 1;").unwrap();
    let b = stmt_def(
        "x",
        [],
        [],
        ty("i32"),
        [
            bound("Clone", [ty("i32")], []),
            bound("Copy", [ty("i32")], []),
        ],
        expr_int("1"),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_where3() {
    let a = Stmt::parse("def x(): i32 where Clone[i32], Copy[i32], = 1;").unwrap();
    let b = stmt_def(
        "x",
        [],
        [],
        ty("i32"),
        [
            bound("Clone", [ty("i32")], []),
            bound("Copy", [ty("i32")], []),
        ],
        expr_int("1"),
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_def_where4() {
    let a = Stmt::parse("def x(): i32 where = { 1 }").unwrap();
    let b = stmt_def("x", [], [], ty("i32"), [], expr_block([], expr_int("1")));
    check!(a, b);
}

#[test]
fn test_parser_program0() {
    let a = Program::parse(
        "def id(x: i32): i32 = x;
         def main(): i32 = id(42);",
    )
    .unwrap();
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
    check!(a, b);
}

#[test]
fn test_parser_stmt_trait0() {
    let a = Stmt::parse("trait Eq[T] {}").unwrap();
    let b = stmt_trait("Eq", ["T"], [], [], []);
    check!(a, b);
}

#[test]
fn test_parser_stmt_trait1() {
    let a = Stmt::parse("trait Eq[T] where Clone[T] {}").unwrap();
    let b = stmt_trait("Eq", ["T"], [bound("Clone", [ty("T")], [])], [], []);
    check!(a, b);
}

#[test]
fn test_parser_stmt_trait2() {
    let a = Stmt::parse(
        "trait Eq[T] {
             def eq(a:T, b:T): bool;
         }",
    )
    .unwrap();
    let b = stmt_trait(
        "Eq",
        ["T"],
        [],
        [tr_def(
            "eq",
            [],
            [("a", ty("T")), ("b", ty("T"))],
            ty("bool"),
            [],
        )],
        [],
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_trait3() {
    let a = Stmt::parse(
        "trait Eq {
             type T[U];
         }",
    )
    .unwrap();
    let b = stmt_trait("Eq", [], [], [], [tr_type("T", ["U"])]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_impl0() {
    let a = Stmt::parse(
        "impl Eq[bool] {
             def eq(x: bool, y: bool): bool = true;
         }",
    )
    .unwrap();
    let b = stmt_impl(
        [],
        head("Eq", [ty("bool")]),
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
    check!(a, b);
}

#[test]
fn test_parser_stmt_impl1() {
    Stmt::parse(
        "impl[T, R] Add[Vec[T], R] where Add[T, R] {
             type Output = Vec[Add[T, R]::Output];
         }",
    )
    .unwrap();
}

#[test]
fn test_parser_stmt_impl2() {
    let a = Stmt::parse("impl Copy[i32] where Clone[i32] {}").unwrap();
    let b = stmt_impl(
        [],
        head("Copy", [ty("i32")]),
        [bound("Clone", [ty("i32")], [])],
        [],
        [],
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_impl3() {
    let a = Stmt::parse("impl Foo[i32] where Bar[i32, f32, T = f32] {}").unwrap();
    let b = stmt_impl(
        [],
        head("Foo", [ty("i32")]),
        [bound("Bar", [ty("i32"), ty("f32")], [("T", ty("f32"))])],
        [],
        [],
    );
    check!(a, b);
}

#[test]
fn test_parser_stmt_var0() {
    let a = Stmt::parse("var x = 1;").unwrap();
    let b = stmt_var("x", ty_hole(), expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_var1() {
    let a = Stmt::parse("var x: i32 = 1;").unwrap();
    let b = stmt_var("x", ty("i32"), expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_var2() {
    let a = Program::parse("var x = 1; var y = x;").unwrap();
    let b = program([
        stmt_var("x", ty_hole(), expr_int("1")),
        stmt_var("y", ty_hole(), expr_var("x")),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_assign0() {
    let a = Expr::parse("x = 1").unwrap();
    let b = expr_assign(expr_var("x"), expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_type0() {
    let a = Stmt::parse("type T = i32;").unwrap();
    let b = stmt_type("T", [], ty("i32"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_type1() {
    let a = Stmt::parse("type T[U] = U;").unwrap();
    let b = stmt_type("T", ["U"], ty("U"));
    check!(a, b);
}

#[test]
fn test_parser_stmt_type2() {
    let a = Stmt::parse("type T[U] = (U, U);").unwrap();
    let b = stmt_type("T", ["U"], ty_tuple([ty("U"), ty("U")]));
    check!(a, b);
}

#[test]
fn test_parser_stmt_struct0() {
    let a = Stmt::parse("struct S;").unwrap();
    let b = stmt_struct("S", [], []);
    check!(a, b);
}

#[test]
fn test_parser_stmt_struct1() {
    let a = Stmt::parse("struct S();").unwrap();
    let b = stmt_struct("S", [], []);
    check!(a, b);
}

#[test]
fn test_parser_stmt_struct2() {
    let a = Stmt::parse("struct S(x:i32);").unwrap();
    let b = stmt_struct("S", [], [("x", ty("i32"))]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_struct3() {
    let a = Stmt::parse("struct S(x:i32);").unwrap();
    let b = stmt_struct("S", [], [("x", ty("i32"))]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_struct4() {
    let a = Stmt::parse("struct S[T](x:T);").unwrap();
    let b = stmt_struct("S", ["T"], [("x", ty("T"))]);
    check!(a, b);
}

#[test]
fn test_parser_expr_struct0() {
    let a = Expr::parse("S(x=1)").unwrap();
    let b = expr_struct("S", [], [("x", expr_int("1"))]);
    check!(a, b);
}

#[test]
fn test_parser_expr_struct1() {
    let a = Expr::parse("S(x=x)").unwrap();
    let b = expr_struct("S", [], [("x", expr_var("x"))]);
    check!(a, b);
}

#[test]
fn test_parser_expr_struct2() {
    let a = Expr::parse("s.x.y").unwrap();
    let b = expr_field(expr_field(expr_var("s"), "x"), "y");
    check!(a, b);
}

// Field punning is done at resolution time.
#[test]
fn test_parser_expr_struct3() {
    let a = Expr::parse("S(x=s.x)").unwrap();
    let b = expr_struct("S", [], [("x", expr_field(expr_var("s"), "x"))]);
    check!(a, b);
}

#[test]
fn test_parser_expr_struct5() {
    let a = Expr::parse("S[i32](x=1)").unwrap();
    let b = expr_struct("S", [ty("i32")], [("x", expr_int("1"))]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_enum0() {
    let a = Stmt::parse("enum E { }").unwrap();
    let b = stmt_enum("E", [], []);
    check!(a, b);
}

#[test]
fn test_parser_stmt_enum1() {
    let a = Stmt::parse("enum E { A(i32) }").unwrap();
    let b = stmt_enum("E", [], [("A", ty("i32"))]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_enum2() {
    let a = Stmt::parse("enum E { A(i32), B(i32) }").unwrap();
    let b = stmt_enum("E", [], [("A", ty("i32")), ("B", ty("i32"))]);
    check!(a, b);
}

#[test]
fn test_parser_expr_enum0() {
    let a = Expr::parse("E::A").unwrap();
    let b = expr_unit_variant("E", [], "A");
    check!(a, b);
}

#[test]
fn test_parser_expr_enum1() {
    let a = Expr::parse("E::A()").unwrap();
    let b = expr_variant("E", [], "A", []);
    check!(a, b);
}

#[test]
fn test_parser_expr_enum2() {
    let a = Expr::parse("E::A(1,)").unwrap();
    let b = expr_variant("E", [], "A", [expr_int("1")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_enum3() {
    let a = Expr::parse("E::A(1, 2)").unwrap();
    let b = expr_variant("E", [], "A", [expr_int("1"), expr_int("2")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_enum4() {
    let a = Expr::parse("E[i32]::A(1, 2)").unwrap();
    let b = expr_variant("E", [ty("i32")], "A", [expr_int("1"), expr_int("2")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_array0() {
    let a = Expr::parse("[1, 2, 3]").unwrap();
    let b = expr_array([expr_int("1"), expr_int("2"), expr_int("3")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_tuple0() {
    let a = Expr::parse("()").unwrap();
    let b = expr_tuple([]);
    check!(a, b);
}

#[test]
fn test_parser_expr_tuple1() {
    let a = Expr::parse("(1,)").unwrap();
    let b = expr_int("1");
    check!(a, b);
}

#[test]
fn test_parser_expr_tuple2() {
    let a = Expr::parse("(1, 2)").unwrap();
    let b = expr_tuple([expr_int("1"), expr_int("2")]);
    check!(a, b);
}

#[test]
fn test_parser_expr_tuple3() {
    let a = Expr::parse("a.0").unwrap();
    let b = expr_index(expr_var("a"), index("0"));
    check!(a, b);
}

#[test]
fn test_parser_program_brace0() {
    let a = Program::parse("{1}").unwrap();
    let b = program([stmt_expr(expr_block([], expr_int("1")))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace1() {
    let a = Program::parse("{1} {2}").unwrap();
    let b = program([
        stmt_expr(expr_block([], expr_int("1"))),
        stmt_expr(expr_block([], expr_int("2"))),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace2() {
    let a = Program::parse("{{1}}").unwrap();
    let b = program([stmt_expr(expr_block([], expr_block([], expr_int("1"))))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace3() {
    let a = Program::parse("{{1} {2}}").unwrap();
    let b = program([stmt_expr(expr_block(
        [stmt_expr(expr_block([], expr_int("1")))],
        expr_block([], expr_int("2")),
    ))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace4() {
    let a = Program::parse("{{1};{2}}").unwrap();
    let b = program([stmt_expr(expr_block(
        [stmt_expr(expr_block([], expr_int("1")))],
        expr_block([], expr_int("2")),
    ))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace5() {
    let a = Program::parse("{{1};{2};}").unwrap();
    let b = program([stmt_expr(expr_block(
        [
            stmt_expr(expr_block([], expr_int("1"))),
            stmt_expr(expr_block([], expr_int("2"))),
        ],
        expr_unit(),
    ))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace6() {
    let a = Program::parse("{;}").unwrap();
    let b = program([stmt_expr(expr_block([], expr_unit()))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace7() {
    let a = Program::parse("{;;;;;;;;}").unwrap();
    let b = program([stmt_expr(expr_block([], expr_unit()))]);
    check!(a, b);
}

#[test]
fn test_parser_program_brace8() {
    let a = Expr::parse("{1;2}").unwrap();
    let b = expr_block([stmt_expr(expr_int("1"))], expr_int("2"));
    check!(a, b);
}

#[test]
fn test_parser_program_paren0() {
    let a = Program::parse("();").unwrap();
    let b = program([stmt_expr(expr_unit())]);
    check!(a, b);
}

#[test]
fn test_parser_program_paren1() {
    let a = Program::parse("(());").unwrap();
    let b = program([stmt_expr(expr_unit())]);
    check!(a, b);
}

#[test]
fn test_parser_program_paren2() {
    let a = Program::parse("({});").unwrap();
    let b = program([stmt_expr(expr_block([], expr_unit()))]);
    check!(a, b);
}

#[test]
fn test_parser_expr_assoc0() {
    let a = Expr::parse("Iterator[Vec[i32]]::next").unwrap();
    let b = expr_assoc("Iterator", [ty_con("Vec", [ty("i32")])], [], "next");
    check!(a, b);
}

#[test]
fn test_parser_type_assoc0() {
    let a = Type::parse("Iterator[Vec[i32]]::Item").unwrap();
    let b = ty_assoc("Iterator", [ty_con("Vec", [ty("i32")])], [], "Item");
    check!(a, b);
}

#[test]
fn test_parser_expr_query0() {
    let a = Expr::parse("from x in [1, 2, 3]").unwrap();
    let b = expr_query([query_from(
        "x",
        expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
    )]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query1() {
    let a = Expr::parse(
        "from x in source()
         select x=f(), y=g()
         into sink()",
    )
    .unwrap();
    let b = expr_query([
        query_from("x", expr_call_direct("source", [], [])),
        query_select([
            ("x", expr_call_direct("f", [], [])),
            ("y", expr_call_direct("g", [], [])),
        ]),
        query_into("sink", [], []),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query2() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select x=1, y=2
         where x > 1",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query3() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select x=1, y=2
         where x > 1
         select x=1, y=2",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
        query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query4() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         var y = f(x)",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_var("y", expr_call_direct("f", [], [expr_var("x")])),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query5() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         group x {
             select x=1, y=2
             var z = f(x)
             where x > 1
         }",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_group(
            expr_var("x"),
            [
                query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
                query_var("z", expr_call_direct("f", [], [expr_var("x")])),
                query_where(expr_gt(expr_var("x"), expr_int("1"))),
            ],
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query6() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         group x {
             compute total = sum of x
         }",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_group(
            expr_var("x"),
            [query_compute("total", expr_var("sum"), expr_var("x"))],
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query7() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         group x {
             compute total = sum of x
             compute lowest = min of x
             compute highest = max of x
         }",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_group(
            expr_var("x"),
            [
                query_compute("total", expr_var("sum"), expr_var("x")),
                query_compute("lowest", expr_var("min"), expr_var("x")),
                query_compute("highest", expr_var("max"), expr_var("x")),
            ],
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query8() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         over tumbling(1) {
             compute total = sum of x
             compute lowest = min of x
             compute highest = max of x
             select x=1, y=2
             where x > 1
         }",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_over(
            expr_call_direct("tumbling", [], [expr_int("1")]),
            [
                query_compute("total", expr_var("sum"), expr_var("x")),
                query_compute("lowest", expr_var("min"), expr_var("x")),
                query_compute("highest", expr_var("max"), expr_var("x")),
                query_select([("x", expr_int("1")), ("y", expr_int("2"))]),
                query_where(expr_gt(expr_var("x"), expr_int("1"))),
            ],
        ),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query9() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select x=1, y=2, z=3
         where x > 1",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_select([
            ("x", expr_int("1")),
            ("y", expr_int("2")),
            ("z", expr_int("3")),
        ]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query10() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         select x=1, y=2, z=3
         where x > 1",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_select([
            ("x", expr_int("1")),
            ("y", expr_int("2")),
            ("z", expr_int("3")),
        ]),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query11() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         from y in [1, 2, 3]
         compute highest = max of x
         compute lowest = min of x
         select x, y, z=3
         var x = f(x)
         var y = g(x)
         where x > 1
         into sink()",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_from(
            "y",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_compute("highest", expr_var("max"), expr_var("x")),
        query_compute("lowest", expr_var("min"), expr_var("x")),
        query_select([
            ("x", expr_var("x")),
            ("y", expr_var("y")),
            ("z", expr_int("3")),
        ]),
        query_var("x", expr_call_direct("f", [], [expr_var("x")])),
        query_var("y", expr_call_direct("g", [], [expr_var("x")])),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
        query_into("sink", [], []),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_expr_query12() {
    let a = Expr::parse(
        "from x in [1, 2, 3]
         from y in [1, 2, 3]
         group (x, y) {
             select x=1, y=2
         }
         into sink()",
    )
    .unwrap();
    let b = expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_from(
            "y",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_group(
            expr_tuple([expr_var("x"), expr_var("y")]),
            [query_select([("x", expr_int("1")), ("y", expr_int("2"))])],
        ),
        query_into("sink", [], []),
    ]);
    check!(a, b);
}

#[test]
fn test_parser_stmt_query() {
    let a = Stmt::parse(
        "from x in [1, 2, 3]
         from y in [1, 2, 3]
         compute highest = max of x
         compute lowest = min of x
         select x, y, z=3
         var x = f(x)
         var y = g(x)
         where x > 1
         into sink();",
    )
    .unwrap();
    let b = stmt_expr(expr_query([
        query_from(
            "x",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_from(
            "y",
            expr_array([expr_int("1"), expr_int("2"), expr_int("3")]),
        ),
        query_compute("highest", expr_var("max"), expr_var("x")),
        query_compute("lowest", expr_var("min"), expr_var("x")),
        query_select([
            ("x", expr_var("x")),
            ("y", expr_var("y")),
            ("z", expr_int("3")),
        ]),
        query_var("x", expr_call_direct("f", [], [expr_var("x")])),
        query_var("y", expr_call_direct("g", [], [expr_var("x")])),
        query_where(expr_gt(expr_var("x"), expr_int("1"))),
        query_into("sink", [], []),
    ]));
    check!(a, b);
}

#[test]
fn test_parser_expr_fun0() {
    let a = Expr::parse("fun() = 1").unwrap();
    let b = expr_fun([], expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_expr_fun1() {
    let a = Expr::parse("fun(x: i32): i32 = 1").unwrap();
    let b = expr_fun_typed([("x", ty("i32"))], ty("i32"), expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_expr_fun2() {
    let a = Expr::parse("fun(x) = fun(y) = 1").unwrap();
    let b = expr_fun(["x"], expr_fun(["y"], expr_int("1")));
    check!(a, b);
}

#[test]
fn test_parser_type_fun0() {
    let a = Type::parse("fun(i32, i32): i32").unwrap();
    let b = ty_fun([ty("i32"), ty("i32")], ty("i32"));
    check!(a, b);
}

#[test]
fn test_parser_type_fun1() {
    let a = Type::parse("fun(i32): fun(i32): i32").unwrap();
    let b = ty_fun([ty("i32")], ty_fun([ty("i32")], ty("i32")));
    check!(a, b);
}

#[test]
fn test_parser_expr_return0() {
    let a = Expr::parse("return 1").unwrap();
    let b = expr_return(expr_int("1"));
    check!(a, b);
}

#[test]
fn test_parser_expr_return1() {
    let a = Expr::parse("return").unwrap();
    let b = expr_return(expr_unit());
    check!(a, b);
}

#[test]
fn test_parser_expr_continue0() {
    let a = Expr::parse("continue").unwrap();
    let b = expr_continue();
    check!(a, b);
}

#[test]
fn test_parser_expr_break0() {
    let a = Expr::parse("break").unwrap();
    let b = expr_break();
    check!(a, b);
}

#[test]
fn test_parser_recover0() {
    let a = Stmt::parse("def f(x: i32): i32 = 1").unwrap_err();
    let b = stmt_err();
    check!(
        a,
        b,
        "Error: Unexpected token `<eof>`
            ╭─[test:1:23]
            │
          1 │ def f(x: i32): i32 = 1
            │                       │
            │                       ╰─ Expected `;`
         ───╯"
    );
}

#[test]
fn test_parser_recover1() {
    let a = Stmt::parse("def f(x: i32): i32 = 1 2;").unwrap_err();
    let b = stmt_def("f", [], [("x", ty("i32"))], ty("i32"), [], expr_int("1"));
    check!(
        a,
        b,
        "Error: Unexpected token `<int>`
            ╭─[test:1:24]
            │
          1 │ def f(x: i32): i32 = 1 2;
            │                        ┬
            │                        ╰── Expected `;`
         ───╯"
    );
}

#[test]
fn test_parser_recover2() {
    let a = Stmt::parse("def f(x: i32): i32 = +;").unwrap_err();
    let b = stmt_def(
        "f",
        [],
        [("x", ty("i32"))],
        ty("i32"),
        [],
        expr_add(expr_err(), expr_err()),
    );
    check!(
        a,
        b,
        "Error: Unexpected token `+`
            ╭─[test:1:22]
            │
          1 │ def f(x: i32): i32 = +;
            │                      ┬
            │                      ╰── Expected one of `{`, `[`, `(`, `-`, `!`, `break`, ...
         ───╯
         
         Error: Unexpected token `;`
            ╭─[test:1:23]
            │
          1 │ def f(x: i32): i32 = +;
            │                       ┬
            │                       ╰── Expected one of `{`, `[`, `(`, `-`, `!`, `break`, ...
         ───╯"
    );
}

#[test]
fn test_parser_recover3() {
    let a = Stmt::parse("def f(x: +): i32 = 1;").unwrap_err();
    let b = stmt_def("f", [], [("x", ty_err())], ty("i32"), [], expr_int("1"));
    check!(
        a,
        b,
        "Error: Unexpected token `+`
            ╭─[test:1:10]
            │
          1 │ def f(x: +): i32 = 1;
            │          ┬
            │          ╰── Expected one of `[`, `(`, `!`, `fun`, `struct`, `<name>`, ...
         ───╯"
    );
}

#[test]
fn test_parser_recover4() {
    let a = Stmt::parse("struct S").unwrap_err();
    let b = stmt_err();
    check!(
        a,
        b,
        "Error: Unexpected token `<eof>`
            ╭─[test:1:9]
            │
          1 │ struct S
            │         │
            │         ╰─ Expected `;`
         ───╯"
    );
}

#[ignore]
#[test]
fn test_parser_depth0() {
    let r = format!("{}{}", "{".repeat(1000), "}".repeat(1000));
    let _ = Expr::parse(&r).unwrap();
}

#[ignore]
#[test]
fn test_parser_depth1() {
    let r = format!("{}", "-1".repeat(10000));
    let _ = Expr::parse(&r).unwrap();
}

#[ignore]
#[test]
fn test_parser_depth2() {
    let r = format!("{}1", "1+".repeat(10000));
    let _ = Expr::parse(&r).unwrap();
}
