mod common;

#[cfg(feature = "optimiser")]
#[test]
fn test_egglog_int() {
    use common::dsl::expr_int;
    use common::dsl::program;
    use common::dsl::stmt_def;
    use common::dsl::types::ty_i32;
    use compiler::opt::into_egg::Context;

    let mut ctx = Context::new();
    let a = ctx.from_program(&program([stmt_def(
        "f",
        [],
        [("x", ty_i32())],
        ty_i32(),
        [],
        expr_int("1").with_type(ty_i32()),
    )]));
    assert_eq!(
        a.to_string(),
        "(Program (StmtListCons (StmtDef (Def \"f\" (TypeListCons (TypeListNil) (TypeCons \"i32\")) (ExprInt 1 i32)))) ProgramNil)"
    );
}
