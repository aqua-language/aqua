pub mod rules;
pub mod lang;
pub mod analysis;
pub mod cost;
use lang::AquaLang;

use egg::*;

fn paren(s: &str) -> String {
    if s.starts_with('(') {
        s.to_string()
    } else {
        format!("({})", s)
    }
}

pub struct Solver(Vec<Rewrite<AquaLang, ()>>);

impl Solver {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn load(mut self, rules: Vec<Rewrite<AquaLang, ()>>) -> Self {
        self.0.extend(rules);
        self
    }

    pub fn check_associativity(&self, expr: &str) -> Result<(), String> {
        let lhs = paren(
            &expr
                .replace("?x", &paren(&expr.replace("?x", "a").replace("?y", "b")))
                .replace("?y", "c"),
        );

        let rhs = paren(
            &expr
                .replace("?x", "a")
                .replace("?y", &paren(&expr.replace("?x", "b").replace("?y", "c"))),
        );

        self.check(&lhs, &rhs)
    }

    // f(x, y) = f(y, x)
    pub fn check_commutativity(&self, expr: &str) -> Result<(), String> {
        let lhs = paren(&expr.replace("?x", "a").replace("?y", "b"));
        let rhs = paren(&expr.replace("?x", "b").replace("?y", "a"));

        self.check(&lhs, &rhs)
    }

    pub fn check(&self, lhs: &str, rhs: &str) -> Result<(), String> {
        println!("Checking {} == {}", lhs, rhs);
        let start = lhs.parse().unwrap();
        let goals = [rhs.parse().unwrap()];

        let runner = Runner::default()
            .with_explanations_enabled()
            .with_expr(&start);

        let id = runner.egraph.find(*runner.roots.last().unwrap());
        let goals_vec = goals.to_vec();

        let mut runner = runner
            .with_hook(move |r| {
                if goals_vec
                    .iter()
                    .all(|g: &Pattern<_>| g.search_eclass(&r.egraph, id).is_some())
                {
                    Err("Proved all goals".into())
                } else {
                    Ok(())
                }
            })
            .run(&self.0);

        for (i, goal) in goals.iter().enumerate() {
            for goal in goals.iter() {
                if let Some(matches) = goal.search_eclass(&runner.egraph, id) {
                    let subst = matches.substs[0].clone();
                    runner = runner.without_explanation_length_optimization();
                    let mut explained = runner.explain_matches(&start, &goal.ast, &subst);
                    explained.get_string_with_let();
                    let flattened = explained.make_flat_explanation().clone();
                    let vanilla_len = flattened.len();
                    explained.check_proof(&self.0);

                    runner = runner.with_explanation_length_optimization();
                    let mut explained_short = runner.explain_matches(&start, &goal.ast, &subst);
                    explained_short.get_string_with_let();
                    let short_len = explained_short.get_flat_strings().len();
                    assert!(short_len <= vanilla_len);
                    explained_short.check_proof(&self.0);
                }
            }
            if goal.search_eclass(&runner.egraph, id).is_none() {
                return Err(format!("Goal {} not proved: {start} != {goal}", i));
            }
        }

        Ok(())
    }
}
