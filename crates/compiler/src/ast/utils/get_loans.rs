use crate::ast::Loan;
use crate::ast::Type;
use crate::traversal::visitor::Visitor;

struct GetLoans {
    loans: Vec<Loan>,
}

impl Type {
    pub fn loans(&self) -> Vec<Loan> {
        let mut get_loans = GetLoans::new();
        get_loans.visit_type(self);
        get_loans.loans
    }
}

impl GetLoans {
    pub fn new() -> GetLoans {
        GetLoans { loans: Vec::new() }
    }
}

impl Visitor for GetLoans {
    fn visit_type(&mut self, t: &Type) {
        if let Type::Ref(loans, _, _) = t {
            self.loans.extend(loans.iter().cloned());
        }
        self._visit_type(t);
    }
}
