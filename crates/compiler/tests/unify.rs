mod common;

use common::dsl::ty;
use common::dsl::ty_con;
use common::dsl::ty_tuple;

use compiler::infer::type_var::TypeVarKind;
use compiler::infer::Context;

#[test]
fn test_unify_atom0() {
    let t0 = ty("i32");
    let t1 = ty("i32");
    let mut ctx = Context::new();
    assert!(ctx.try_unify(&t0, &t1).is_ok());
}

#[test]
fn test_unify_atom1() {
    let t0 = ty("i32");
    let t1 = ty("i64");
    let mut ctx = Context::new();
    assert_eq!(ctx.try_unify(&t0, &t1), Err(()));
}

#[test]
fn test_unify_var0() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ty("i32");
    assert!(ctx.try_unify(&t0, &t1).is_ok());
}

#[test]
fn test_unify_var1() {
    let mut ctx = Context::new();
    let t0 = ty("i32");
    let t1 = ctx.fresh(TypeVarKind::General);
    assert!(ctx.try_unify(&t0, &t1).is_ok());
}

#[test]
fn test_unify_var2() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ctx.fresh(TypeVarKind::General);
    assert!(ctx.try_unify(&t0, &t1).is_ok());
}

#[test]
fn test_unify_var3() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ctx.fresh(TypeVarKind::General);
    assert!(ctx.try_unify(&t0, &ty("i32")).is_ok());
    assert!(ctx.try_unify(&t1, &ty("i32")).is_ok());
    assert!(ctx.try_unify(&t0, &t1).is_ok());
}

#[test]
fn test_unify_tc0() {
    let mut ctx = Context::new();
    let t1 = ty_con("Vec", [ctx.fresh(TypeVarKind::General)]);
    let t2 = ty_con("Vec", [ty("i32")]);
    assert!(ctx.try_unify(&t1, &t2).is_ok());
}

#[test]
fn test_unify_tc1() {
    let mut ctx = Context::new();
    let t0 = ty_con("Vec", [ctx.fresh(TypeVarKind::General)]);
    let t1 = ty_con("Vec", [ctx.fresh(TypeVarKind::General)]);
    assert!(ctx.try_unify(&t0, &t1).is_ok());
}

#[test]
fn test_unify_tc2() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ctx.fresh(TypeVarKind::General);
    let t2 = ty_con("Vec", [ty_con("Vec", [t0.clone()])]);
    let t3 = ty_con("Vec", [ty_con("Vec", [t1])]);
    assert!(ctx.try_unify(&t0, &ty("i32")).is_ok());
    assert!(ctx.try_unify(&t2, &t3).is_ok());
}

#[test]
fn test_unify_tc3() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ty_con("Vec", [t0.clone()]);
    let t2 = ty_con("Vec", [ty_con("Vec", [ty("i32")])]);
    assert!(ctx.try_unify(&t0, &ty_con("Vec", [ty("i32")])).is_ok());
    assert!(ctx.try_unify(&t1, &t2).is_ok());
}

#[test]
fn test_unify_tc4() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    assert!(ctx.try_unify(&t0, &ty("i32")).is_ok());
    let t1 = ty_con("Vec", [t0.clone()]);
    let t2 = ty_con("Vec", [ty("i64")]);
    assert_eq!(ctx.try_unify(&t1, &t2), Err(()));
}

#[test]
fn test_union_find1() {
    let mut ctx = Context::new();
    let t0 = ty_tuple([ty("bool"), ty("i32")]);
    let t1 = ty_tuple([ctx.fresh(TypeVarKind::General), ty("i32")]);

    ctx.try_unify(&t0, &t1).unwrap();
}

#[test]
fn test_union_find2() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ty_tuple([ty("bool"), ty("i32")]);
    let t2 = ty_tuple([t0.clone(), ty("i32")]);

    assert!(ctx.try_unify(&t1, &t2).is_ok());
    assert!(ctx.try_unify(&t0, &ty("i32")).is_ok());
}

#[test]
fn test_union_find3() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ctx.fresh(TypeVarKind::General);

    assert!(ctx.try_unify(&t0, &t1).is_ok());
    assert!(ctx.try_unify(&t0, &ty("i32")).is_ok());
    assert_eq!(t1.apply(&mut ctx), ty_tuple([ty("i32"), ty("bool")]));
}

#[test]
fn test_union_find4() {
    let mut ctx = Context::new();
    let t0 = ctx.fresh(TypeVarKind::General);
    let t1 = ctx.fresh(TypeVarKind::General);

    assert!(ctx
        .try_unify(&t0, &ty_tuple([ty("i32"), ty("bool")]))
        .is_ok());
    assert!(ctx.try_unify(&t0, &t1).is_ok());
    assert_eq!(t1.apply(&mut ctx), ty_tuple([ty("i32"), ty("bool")]));
}
