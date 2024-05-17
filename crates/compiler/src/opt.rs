use egglog::ast::Expr;
use egglog::ast::GenericAction;
use egglog::ast::GenericCommand;
use egglog::ast::GenericFact;
use egglog::ast::GenericRunConfig;
use egglog::ast::GenericSchedule;
use symbol_table::GlobalSymbol;

const LIB: &str = include_str!("opt/lib.egg");

pub struct Optimiser {
    egraph: egglog::EGraph,
    desugar: egglog::ast::desugar::Desugar,
}

impl Optimiser {
    pub fn new() -> Self {
        let egraph = egglog::EGraph::default();
        let desugar = egglog::ast::desugar::Desugar::default();
        let mut this = Self { egraph, desugar };
        this.include(LIB).unwrap();
        this
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
}
