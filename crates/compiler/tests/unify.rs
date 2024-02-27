use compiler::infer::unify;
use compiler::util::ty;
use compiler::util::ty_con;
use compiler::util::ty_var;

#[test]
fn test_unify_atom0() {
    let mut sub = vec![];
    let t0 = ty("i32");
    let t1 = ty("i32");
    assert!(unify(&mut sub, &t0, &t1).is_ok());
    assert!(sub.is_empty());
}

#[test]
fn test_unify_atom1() {
    let mut sub = vec![];
    let t0 = ty("i32");
    let t1 = ty("i64");
    assert_eq!(unify(&mut sub, &t0, &t1), Err((ty("i32"), ty("i64"))));
    assert!(sub.is_empty());
}

#[test]
fn test_unify_var0() {
    let mut sub = vec![];
    let t0 = ty_var("?T");
    let t1 = ty("i32");
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_var1() {
    let mut sub = vec![];
    let t0 = ty("i32");
    let t1 = ty_var("?T");
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_var2() {
    let mut sub = vec![];
    let t0 = ty_var("?T");
    let t1 = ty_var("?U");
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_var3() {
    let mut sub = vec![("T".into(), ty("i32")), ("U".into(), ty("i32"))];
    let t0 = ty_var("?T");
    let t1 = ty_var("?U");
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_tc0() {
    let mut sub = vec![("T".into(), ty("i32"))];
    let t0 = ty_con("Vec", [ty_var("?T")]);
    let t1 = ty_con("Vec", [ty("i32")]);
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_tc1() {
    let mut sub = Vec::new();
    let t0 = ty_con("Vec", [ty_var("?T")]);
    let t1 = ty_con("Vec", [ty_var("?U")]);
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_tc2() {
    let mut sub = vec![("T".into(), ty("i32"))];
    let t0 = ty_con("Vec", [ty_con("Vec", [ty_var("?T")])]);
    let t1 = ty_con("Vec", [ty_con("Vec", [ty_var("?U")])]);
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_tc3() {
    let mut sub = vec![("T".into(), ty_con("Vec", [ty("i32")]))];
    let t0 = ty_con("Vec", [ty_var("?T")]);
    let t1 = ty_con("Vec", [ty_con("Vec", [ty("i32")])]);
    assert!(unify(&mut sub, &t0, &t1).is_ok());
}

#[test]
fn test_unify_tc4() {
    let mut sub = vec![("?T".into(), ty("i32"))];
    let t0 = ty_con("Vec", [ty_var("?T")]);
    let t1 = ty_con("Vec", [ty("i64")]);
    assert_eq!(
        unify(&mut sub, &t0, &t1),
        Err((ty_con("Vec", [ty("i32")]), t1))
    );
}
