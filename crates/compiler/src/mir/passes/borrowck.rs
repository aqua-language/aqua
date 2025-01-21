use crate::ast::Loan;
use crate::ast::Place;
use crate::mir::Function;
use crate::mir::Operation;
use crate::mir::Rvalue;

pub struct Context<'a> {
    function: &'a Function,
}

impl<'a> Context<'a> {
    pub fn new(function: &'a Function) -> Context<'a> {
        Context { function }
    }

    fn check(&self) {
        for block in &self.function.blocks {
            for stmt in &block.stmts {
                match &stmt.op {
                    Operation::Assign(lhs, rhs) => {
                        let loan = Loan {
                            place: lhs.clone(),
                            mutable: true,
                        };
                        if !self.permits(&stmt.live_out, &loan) {
                            panic!("Borrowck error");
                        }
                        match rhs {
                            Rvalue::Use(_) => {}
                            Rvalue::Ref { mutable, place } => {
                                let loan = Loan {
                                    place: place.clone(),
                                    mutable: *mutable,
                                };
                                if !self.permits(&stmt.live_in, &loan) {
                                    panic!("Borrowck error");
                                }
                            }
                        }
                    }
                    Operation::StorageLive(..) => {}
                    Operation::StorageDead(..) => {}
                    Operation::Call { dest, .. } => {
                        let loan = Loan {
                            place: dest.clone(),
                            mutable: true,
                        };
                        if !self.permits(&stmt.live_in, &loan) {
                            panic!("Borrowck error");
                        }
                    }
                    Operation::Noop => {}
                }
            }
        }
    }

    fn permits(&self, live_out: &[Place], loan1: &Loan) -> bool {
        for place in live_out {
            for loan2 in place.local.ty.loans() {
                if !self.compatible(loan1, &loan2) {
                    return false;
                }
            }
        }
        true
    }

    fn compatible(&self, loan1: &Loan, loan2: &Loan) -> bool {
        (!loan1.mutable && !loan2.mutable) || self.disjoint(&loan1.place, &loan2.place)
    }

    fn disjoint(&self, p1: &Place, p2: &Place) -> bool {
        !p1.is_prefix_of(p2) && !p2.is_prefix_of(p1)
    }
}

impl Function {
    pub fn borrowck(&self) {
        Context::new(self).check();
    }
}
