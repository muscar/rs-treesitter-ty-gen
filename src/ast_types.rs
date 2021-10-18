use std::fmt::{self, Display};

use crate::grammar::RuleBody;

#[derive(Debug)]
pub struct AstType {
    name: String,
    repr: AstTypeRepr,
}

#[derive(Debug)]
enum AstTypeRepr {
    Sum(Vec<(String, AstTypeRepr)>),
    Product(Vec<(Option<String>, AstTypeRepr)>),
    Ctor(String, Vec<AstTypeRepr>),
    Name(String),
}

impl AstType {
    pub fn from_rule(name: &str, rule: &RuleBody) -> Self {
        AstType {
            name: name.to_owned(),
            repr: AstTypeRepr::from_rule_body(name, rule),
        }
    }
}

impl Display for AstType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.repr)
    }
}

impl AstTypeRepr {
    fn from_rule_body(name: &str, rule: &RuleBody) -> Self {
        match rule {
            RuleBody::Repeat { content } => AstTypeRepr::Ctor(
                "list".to_owned(),
                vec![AstTypeRepr::from_rule_body(name, &*content)],
            ),
            RuleBody::Choice { members } => AstTypeRepr::Sum(
                members
                    .iter()
                    .enumerate()
                    .map(|(_, r)| {
                        (
                            format!("{}_CTOR", name.to_uppercase()),
                            AstTypeRepr::from_rule_body(name, r),
                        )
                    })
                    .collect(),
            ),
            RuleBody::Seq { members } => AstTypeRepr::Product(
                members
                    .iter()
                    .enumerate()
                    .map(|(_, r)| (None, AstTypeRepr::from_rule_body(name, r)))
                    .collect(),
            ),
            RuleBody::PrecLeft { content } => AstTypeRepr::from_rule_body(name, &*content),
            RuleBody::Symbol { name } => AstTypeRepr::Name(name.to_owned()),
            RuleBody::String { .. } | RuleBody::Pattern { .. } => {
                AstTypeRepr::Name("string".to_owned())
            }
        }
    }
}

impl Display for AstTypeRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstTypeRepr::Sum(cases) => {
                for (i, (name, c)) in cases.iter().enumerate() {
                    write!(f, "\n | {}_{} ({})", name, i, c)?;
                }
            }
            AstTypeRepr::Product(members) => {
                write!(f, "(")?;
                let mut it = members.iter();
                if let Some((_, x)) = it.next() {
                    x.fmt(f)?;
                    for (_, t) in it {
                        write!(f, ", ")?;
                        t.fmt(f)?;
                    }
                }
                write!(f, ")")?;
            }
            AstTypeRepr::Ctor(name, args) => {
                write!(f, "{}(", name)?;
                let mut it = args.iter();
                if let Some(x) = it.next() {
                    x.fmt(f)?;
                    for t in it {
                        write!(f, ", ")?;
                        t.fmt(f)?;
                    }
                }
                write!(f, ")")?;
            }
            AstTypeRepr::Name(name) => write!(f, "{}", name)?,
        }
        Ok(())
    }
}
