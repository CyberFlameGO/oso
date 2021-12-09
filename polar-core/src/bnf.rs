use crate::{
    terms::*,
};

impl Term {
    fn negated(mut self) -> Self {
        use {Value::*, Operator::*};
        match self.value() {
            Expression(Operation { operator, args }) => match operator {
                Not => {
                    assert_eq!(args.len(), 1);
                    self.replace_value(args[0].value().clone())
                }
                And => {
                    assert_eq!(args.len(), 2);
                    self.replace_value(Expression(Operation {
                        operator: Or,
                        args: args.iter().cloned().map(|t| t.negated()).collect(),
                    }))
                }
                Or => {
                    assert_eq!(args.len(), 2);
                    self.replace_value(Expression(Operation {
                        operator: And,
                        args: args.iter().cloned().map(|t| t.negated()).collect(),
                    }))
                }
                _ => self.replace_value(Expression(op!(Not, self.clone()))),
            }
            Boolean(b) => self.replace_value(Boolean(!b)),
            _ => self.replace_value(Expression(op!(Not, self.clone()))),
        }
        self
    }

    /// negation normal form -- all `not` operators are nested inside of `and`/`or`
    /// using De Morgan's law
    pub fn nnf(self) -> Self {
        use Operator::*;
        match self.value().as_expression() {
            Ok(Operation { operator: Not, args }) => args[0].clone().nnf().negated(),
            Ok(Operation { operator, args }) =>
                self.clone_with_value(Value::Expression(Operation {
                    operator: *operator,
                    args: args.iter().cloned().map(|t| t.nnf()).collect(),
                })),
            _ => self,
        }
    }


    fn lhs(&self) -> &Self {
        &self.value().as_expression().unwrap().args[0]
    }

    fn rhs(&self) -> &Self {
        &self.value().as_expression().unwrap().args[1]
    }

    pub fn dnf(self) -> Self { self.nnf().dnf_inner() }
    fn dnf_inner(mut self) -> Self {
        use {Value::*, Operator::*};
        match self.value().as_expression() {
            Ok(Operation { operator, args }) => {
                let args: Vec<_> = args.iter().cloned().map(|t| t.dnf_inner()).collect();
                let operator = *operator;
                if operator == And && args[0].is_or() {
                    or_(and_(args[0].lhs().clone(), args[1].clone()),
                        and_(args[0].rhs().clone(), args[1].clone()))
                        .dnf_inner()
                } else if operator == And && args[1].is_or() {
                    or_(and_(args[0].clone(), args[1].lhs().clone()),
                        and_(args[0].clone(), args[1].rhs().clone()))
                        .dnf_inner()
                } else {
                    self.replace_value(Expression(Operation { operator, args }));
                    self
                }
            }
            _ => self,
        }
    }

    pub fn cnf(self) -> Self { self.nnf().cnf_inner() }
    fn cnf_inner(mut self) -> Self {
        use {Value::*, Operator::*};
        match self.value().as_expression() {
            Ok(Operation { operator, args }) => {
                let args: Vec<_> = args.iter().cloned().map(|t| t.cnf_inner()).collect();
                let operator = *operator;
                if operator == Or && args[0].is_and() {
                    and_(or_(args[0].lhs().clone(), args[1].clone()),
                         or_(args[0].rhs().clone(), args[1].clone()))
                        .cnf_inner()
                } else if operator == Or && args[1].is_and() {
                    and_(or_(args[0].clone(), args[1].lhs().clone()),
                         or_(args[0].clone(), args[1].rhs().clone()))
                        .cnf_inner()
                } else {
                    self.replace_value(Expression(Operation { operator, args }));
                    self
                }
            }
            _ => self,
        }
    }

    fn is_and(&self) -> bool {
        matches!(self.value().as_expression(), Ok(Operation { operator: Operator::And, ..}))
    }
    fn is_or(&self) -> bool {
        matches!(self.value().as_expression(), Ok(Operation { operator: Operator::Or, ..}))
    }
}


fn or_(l: Term, r: Term) -> Term { term!(op!(Or, l, r)) }
fn and_(l: Term, r: Term) -> Term { term!(op!(And, l, r)) }
fn not_(t: Term) -> Term { term!(op!(Not, t)) }
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nnf() {
        let ex =
            or_(not_(var!("p")),
                and_(var!("q"),
                     not_(and_(not_(var!("r")),
                               var!("s")))));

        assert_eq!(ex.clone().nnf(), ex.clone().nnf().nnf());

        let nnf =
            or_(not_(var!("p")),
                and_(var!("q"),
                     or_(var!("r"),
                         not_(var!("s")))));

        assert_eq!(nnf, ex.nnf());
    }

    #[test]
    fn test_dnf() {
        let ex =
            or_(not_(var!("p")),
                and_(var!("q"),
                     not_(and_(not_(var!("r")), var!("s")))));

        assert_eq!(ex.clone().dnf(), ex.clone().dnf().dnf());

        let dnf =
            or_(not_(var!("p")),
                or_(and_(var!("q"), var!("r")),
                    and_(var!("q"), not_(var!("s")))));

        assert_eq!(dnf, ex.dnf());
    }

    #[test]
    fn test_cnf() {
        let ex =
            or_(not_(var!("p")),
                and_(var!("q"),
                     not_(and_(not_(var!("r")), var!("s")))));

        assert_eq!(ex.clone().cnf(), ex.clone().cnf().cnf());

        let cnf =
            and_(or_(not_(var!("p")), var!("q")),
                 or_(not_(var!("p")),
                     or_(var!("r"), not_(var!("s")))));

        assert_eq!(cnf.to_polar(), ex.cnf().to_polar());
    }
}
