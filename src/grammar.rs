use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::name_gen::NameGen;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawGrammar {
    pub name: String,
    rules: HashMap<String, RuleBody>,
    extras: Vec<RuleBody>,
}

pub struct Grammar {
    pub name: String,
    rules: Vec<Rule>,
}

impl Grammar {
    pub fn from_file<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let s = fs::read_to_string(path).expect("failed to open grammar file");
        let raw_grammar: RawGrammar =
            serde_json::from_str(&s).expect("failed to deserialize grammar");
        let extras = raw_grammar
            .extras
            .iter()
            .filter_map(|r| match r {
                RuleBody::Symbol { name } => Some(name.to_owned()),
                _ => None,
            })
            .collect::<HashSet<String>>();
        Self {
            name: raw_grammar.name,
            rules: raw_grammar
                .rules
                .iter()
                .map(|(name, body)| Rule {
                    name: name.clone(),
                    body: body.clone(),
                    is_extra: extras.contains(name),
                })
                .collect(),
        }
    }

    pub fn get_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }
}

#[derive(Clone)]
pub struct Rule {
    pub name: String,
    pub body: RuleBody,
    pub is_extra: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleBody {
    Repeat { content: Box<RuleBody> },
    Choice { members: Vec<RuleBody> },
    Seq { members: Vec<RuleBody> },
    PrecLeft { content: Box<RuleBody> },
    PrecRight { content: Box<RuleBody> },
    Symbol { name: String },
    String { value: String },
    Pattern { value: String },
}

impl RuleBody {
    pub fn get_nonterminals(&self) -> Vec<String> {
        match self {
            RuleBody::Repeat { content }
            | RuleBody::PrecLeft { content }
            | RuleBody::PrecRight { content } => {
                if let RuleBody::Symbol { name } = &**content {
                    vec![name.clone()]
                } else {
                    vec![]
                }
            }
            RuleBody::Choice { members } | RuleBody::Seq { members } => members
                .iter()
                .filter_map(|b| match b {
                    RuleBody::Symbol { name } => Some(name.clone()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    }

    pub fn map_subexps<F, T: Default>(&self, mut f: F) -> (RuleBody, T)
    where
        F: FnMut(&[RuleBody]) -> (Vec<RuleBody>, T),
    {
        match self {
            RuleBody::Repeat { content } => {
                let (new_content, data) = f(&[*content.clone()]);
                (
                    RuleBody::Repeat {
                        content: Box::new(new_content[0].clone()),
                    },
                    data,
                )
            }
            RuleBody::Choice { members } => {
                let (new_members, data) = f(&members[..]);
                (
                    RuleBody::Choice {
                        members: new_members,
                    },
                    data,
                )
            }
            RuleBody::Seq { members } => {
                let (new_members, data) = f(&members[..]);
                (
                    RuleBody::Seq {
                        members: new_members,
                    },
                    data,
                )
            }
            RuleBody::PrecLeft { content } | RuleBody::PrecRight { content } => {
                content.map_subexps(f)
            }
            _ => (self.clone(), Default::default()),
        }
    }

    pub fn hoist_subexps<P>(
        &self,
        name: &str,
        pred: P,
        gen: &mut NameGen,
    ) -> (RuleBody, Vec<(String, RuleBody)>)
    where
        P: Fn(&RuleBody) -> bool,
    {
        self.map_subexps(|rules| {
            let mut subexps = Vec::new();
            let mut new_rules = Vec::new();
            for r in rules {
                let new_r = if pred(r) {
                    let fresh_name = gen.get_fresh_name(name);
                    subexps.push((fresh_name.clone(), r.clone()));
                    RuleBody::Symbol { name: fresh_name }
                } else {
                    r.clone()
                };
                new_rules.push(new_r);
            }
            (new_rules, subexps)
        })
    }
}
