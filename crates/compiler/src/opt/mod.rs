use egglog::ast::Expr;
use egglog::ast::GenericAction;
use egglog::ast::GenericCommand;
use egglog::ast::GenericFact;
use egglog::ast::GenericRunConfig;
use egglog::ast::GenericSchedule;
use egglog::Term;
use egglog::TermDag;
use symbol_table::GlobalSymbol;

pub mod from_egg;
pub mod into_egg;

const LIB: &str = include_str!("lib.egg");

pub struct Optimiser {
    egraph: egglog::EGraph,
    desugar: egglog::ast::desugar::Desugar,
}

impl Default for Optimiser {
    fn default() -> Self {
        let egraph = egglog::EGraph::default();
        let desugar = egglog::ast::desugar::Desugar::default();
        let mut this = Self { egraph, desugar };
        this.include(LIB).unwrap();
        this
    }
}

impl Optimiser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include(&mut self, s: &str) -> Result<Vec<String>, egglog::Error> {
        let program = self.desugar.parse_program(s)?;
        self.egraph.run_program(program)
    }

    pub fn check(&mut self, a: Expr, b: Expr) -> bool {
        let x0 = self.desugar.get_fresh();
        let x1 = self.desugar.get_fresh();
        match self.egraph.run_program(
            [
                GenericCommand::Action(GenericAction::Let((), x0, a.clone())),
                GenericCommand::Action(GenericAction::Let((), x1, b.clone())),
                GenericCommand::RunSchedule(
                    GenericSchedule::Run(GenericRunConfig {
                        ruleset: GlobalSymbol::new("opt"),
                        until: None,
                    })
                    .saturate(),
                ),
                GenericCommand::Check(vec![GenericFact::Eq(vec![a, b])]),
            ]
            .into_iter()
            .collect(),
        ) {
            Ok(_) => true,
            Err(egglog::Error::CheckError(_)) => false,
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    pub fn opt(&mut self, e: Expr) -> (TermDag, Term) {
        let x = "_0".into();
        self.egraph
            .run_program(vec![
                GenericCommand::Action(GenericAction::Let((), x, e)),
                GenericCommand::RunSchedule(
                    GenericSchedule::Run(GenericRunConfig {
                        ruleset: "".into(),
                        until: None,
                    })
                    .saturate(),
                ),
            ])
            .unwrap();
        let (sort, value) = self
            .egraph
            .eval_expr(&egglog::ast::Expr::Var((), x.into()))
            .unwrap();
        let mut termdag = TermDag::default();
        let (_, term) = self.egraph.extract(value, &mut termdag, &sort);
        (termdag, term)
    }

    pub fn transaction<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.egraph
            .run_program(vec![GenericCommand::Push(1)])
            .unwrap();
        let v = f(self);
        self.egraph
            .run_program(vec![GenericCommand::Pop(1)])
            .unwrap();
        v
    }

}
