use opt::rules;
use opt::Solver;

// f(?x,?y) = ?x * ?y
#[test]
fn test_product() {
    let f = "(mul ?x ?y)";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_associativity(f).is_ok());
    assert!(solver.check_commutativity(f).is_ok());
}

#[test]
fn test_division() {
    let f = "(div ?x ?y)";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_associativity(f).is_err());
    assert!(solver.check_commutativity(f).is_err());
}

#[test]
fn test_subtraction() {
    let f = "(sub ?x ?y)";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_associativity(f).is_err());
    assert!(solver.check_commutativity(f).is_err());
}

// sqrt(?x^2) = ?x
// sqrt(?x)^2 = ?x
#[test]
fn test_sqrt_pow() {
    let f0 = "(sqrt (pow ?x 2))";
    let f1 = "(pow (sqrt ?x) 2)";
    let solver = Solver::new().load(rules::arithmetic()).load(rules::power());

    assert!(solver.check(f0, "?x").is_ok());
    assert!(solver.check(f1, "?x").is_ok());
}

// f(?x,?y) = ?x + ?y
#[test]
fn test_sum() {
    let f = "(add ?x ?y)";
    let solver = Solver::new().load(rules::arithmetic());

    assert!(solver.check_associativity(f).is_ok());
    assert!(solver.check_commutativity(f).is_ok());
}
