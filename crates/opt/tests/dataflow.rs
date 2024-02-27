use opt::rules;
use opt::Solver;

#[test]
fn test_fold_fold() {
    let lhs = "(pair (fold ?x ?init0 ?combine0) (fold ?x ?init1 ?combine1))";
    let rhs = "(fold
                 ?x
                 (fun elem (pair (call ?init0 elem) (call ?init1 elem)))
                 (fun acc (fun elem
                     (pair
                         (call (call ?combine0 (fst acc)) elem)
                         (call (call ?combine1 (snd acc)) elem)))))";
    let s = Solver::new().load(rules::fold()).load(rules::function());

    assert!(s.check(lhs, rhs).is_ok());
}
