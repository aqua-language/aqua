// Commutative = We can keep one partial aggregate
// Associative = We can keep a tree of partial aggregates

// These tests verify algebraic properties of advanced UDAFs (User Defined Aggregate Functions)
// that contain composite expressions. These tests use an algebraic solver to rewrite the
// expressions to verify if they are commutative and associative.

use opt::rules;
use opt::Solver;

// f(?x,?y) = ?x + ?y + 2
// f(?y,?x) = ?y + ?x + 2
#[test]
fn test_sum_const() {
    let f = "(add (add ?x ?y) 2)";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_associativity(f).is_ok());
    assert!(solver.check_commutativity(f).is_ok());
}

// f(?x,?y) = ?x^2 + ?y^2
#[test]
fn test_pow_sum() {
    let f = "(add (pow ?x 2) (pow ?y 2))";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_commutativity(f).is_ok());
    assert!(solver.check_associativity(f).is_err());
}

// f(?x,?y) = sqrt(?x^2 + ?y^2)
#[test]
fn test_euclidean_norm() {
    let f = "(sqrt (add (pow ?x 2) (pow ?y 2)))";
    let solver = Solver::new().load(rules::arithmetic()).load(rules::power());

    assert!(solver.check_commutativity(f).is_ok());
    assert!(solver.check_associativity(f).is_ok());
}

// f(?x,?y) = ?x * ?y * 2
#[test]
fn test_product_const() {
    let f = "(mul (mul ?x ?y) 2)";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_associativity(f).is_ok());
    assert!(solver.check_commutativity(f).is_ok());
}

// Trigonometric functions
// f(?x,?y) = sin(?x) + sin(?y)
#[test]
fn test_sin_sum() {
    let f = "(add (sin ?x) (sin ?y))";
    let solver = Solver::new()
        .load(rules::arithmetic())
        .load(rules::trigonometric());

    assert!(solver.check_commutativity(f).is_ok());
    assert!(solver.check_associativity(f).is_err());
}

#[test]
fn test_cos_sum() {
    let f = "(add (cos ?x) (cos ?y))";
    let solver = Solver::new()
        .load(rules::arithmetic())
        .load(rules::trigonometric());

    assert!(solver.check_commutativity(f).is_ok());
    assert!(solver.check_associativity(f).is_err());
}

#[test]
fn test_cond_min() {
    let f = "(if (le ?x ?y) ?x ?y)";
    let solver = Solver::new()
        .load(rules::relational())
        .load(rules::if_else());
    assert!(solver.check_commutativity(f).is_ok());
    // assert!(solver.check_associativity(f).is_ok());
}

#[test]
fn test_cond_max() {
    let f = "(if (le ?x ?y) ?y ?x)";
    let solver = Solver::new()
        .load(rules::relational())
        .load(rules::if_else());

    assert!(solver.check_commutativity(f).is_ok());
    // assert!(solver.check_associativity(f).is_ok());
}

#[test]
fn test_min() {
    let f = "(min ?x ?y)";
    let solver = Solver::new().load(rules::min_max());

    assert!(solver.check_commutativity(f).is_ok());
    assert!(solver.check_associativity(f).is_ok());
}

#[test]
fn test_max() {
    let f = "(max ?x ?y)";
    let solver = Solver::new().load(rules::min_max());

    assert!(solver.check_commutativity(f).is_ok());
    assert!(solver.check_associativity(f).is_ok());
}

// #[test]
// fn test_sum_vec() {
//     todo!()
// }

#[test]
fn test_fun() {
    let solver = Solver::new().load(rules::function());

    assert!(solver.check("(fun ?a ?b)", "(fun ?x ?y)").is_ok());
}

#[test]
fn test_fun_call() {
    let solver = Solver::new(); //.load(rules::function());

    assert!(solver.check("?x", "?x").is_ok());
}

#[test]
fn test_stddev() {
    let solver = Solver::new().load(rules::arithmetic()).load(rules::power());
    assert!(solver.check_associativity("(fold)").is_ok());
}
