#![feature(box_patterns)]
mod graph;

use graph::{Graph, VertexId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    fs,
};

struct NameGen {
    idx: usize,
}

impl NameGen {
    fn new() -> Self {
        Self { idx: 0 }
    }

    fn get_fresh_name(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.idx);
        self.idx += 1;
        name
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Grammar {
    name: String,
    rules: HashMap<String, RuleBody>,
    extras: Vec<RuleBody>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RuleBody {
    Repeat { content: Box<RuleBody> },
    Choice { members: Vec<RuleBody> },
    Seq { members: Vec<RuleBody> },
    PrecLeft { content: Box<RuleBody> },
    Symbol { name: String },
    String { value: String },
    Pattern { value: String },
}

impl RuleBody {
    fn is_terminal(&self) -> bool {
        matches!(
            self,
            RuleBody::Symbol { .. } | RuleBody::String { .. } | RuleBody::Pattern { .. }
        )
    }

    fn hoist_subexpressions(
        &self,
        name: &str,
        gen: &mut NameGen,
    ) -> (RuleBody, Vec<(String, RuleBody)>) {
        let mut subexps = Vec::new();
        let result = match self {
            RuleBody::Repeat { content } => {
                if matches!(content, box RuleBody::Choice { .. }) {
                    let fresh_name = gen.get_fresh_name(name);
                    subexps.push((fresh_name.clone(), *content.clone()));
                    RuleBody::Repeat {
                        content: Box::new(RuleBody::Symbol { name: fresh_name }),
                    }
                } else {
                    self.clone()
                }
            }
            RuleBody::Choice { members } => {
                let mut new_members = Vec::new();
                for b in members {
                    let new_b = if matches!(b, RuleBody::Choice { .. }) {
                        let fresh_name = gen.get_fresh_name(name);
                        subexps.push((fresh_name.clone(), b.clone()));
                        RuleBody::Symbol { name: fresh_name }
                    } else {
                        b.clone()
                    };
                    new_members.push(new_b);
                }
                RuleBody::Choice {
                    members: new_members,
                }
            }
            RuleBody::Seq { members } => {
                let mut new_members = Vec::new();
                for b in members {
                    let new_b = if matches!(b, RuleBody::Choice { .. }) {
                        let fresh_name = gen.get_fresh_name(name);
                        subexps.push((fresh_name.clone(), b.clone()));
                        RuleBody::Symbol { name: fresh_name }
                    } else {
                        b.clone()
                    };
                    new_members.push(new_b);
                }
                RuleBody::Seq {
                    members: new_members,
                }
            }
            RuleBody::PrecLeft { content } => *content.clone(),
            _ => self.clone(),
        };
        (result, subexps)
    }

    fn get_nonterminals(&self) -> Vec<String> {
        match self {
            RuleBody::Repeat {
                content: box RuleBody::Symbol { name: symbol_name },
            } => vec![symbol_name.to_owned()],
            RuleBody::Choice { members } | RuleBody::Seq { members } => members
                .iter()
                .filter_map(|b| match b {
                    RuleBody::Symbol { name } => Some(name.clone()),
                    _ => None,
                })
                .collect(),
            RuleBody::PrecLeft {
                content: box RuleBody::Symbol { name: symbol_name },
            } => vec![symbol_name.to_owned()],
            _ => vec![],
        }
    }
}

#[derive(Debug)]
struct AstType {
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
    fn from_rule(name: &str, rule: &RuleBody) -> Self {
        AstType {
            name: name.to_owned(),
            repr: AstTypeRepr::from_rule_body(name, rule),
        }
    }
}

impl Display for AstType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                    std::fmt::Display::fmt(&x, f)?;
                    for (_, t) in it {
                        write!(f, ", ")?;
                        std::fmt::Display::fmt(&t, f)?;
                    }
                }
                write!(f, ")")?;
            }
            AstTypeRepr::Ctor(name, args) => {
                write!(f, "{}(", name)?;
                let mut it = args.iter();
                if let Some(x) = it.next() {
                    std::fmt::Display::fmt(&x, f)?;
                    for t in it {
                        write!(f, ", ")?;
                        std::fmt::Display::fmt(&t, f)?;
                    }
                }
                write!(f, ")")?;
            }
            AstTypeRepr::Name(name) => write!(f, "{}", name)?,
        }
        Ok(())
    }
}

struct TypeGenerator {
    graph: Graph<String, usize>,
    vertex_map: HashMap<String, VertexId>,
    rules: HashMap<String, RuleBody>,
    gen: NameGen,
}

impl TypeGenerator {
    fn new() -> Self {
        Self {
            graph: Graph::new(),
            vertex_map: HashMap::new(),
            rules: HashMap::new(),
            gen: NameGen::new(),
        }
    }

    fn get_or_insert_vertex(&mut self, name: &str, weight: usize) -> VertexId {
        if !self.vertex_map.contains_key(name) {
            let id = self.graph.add_vertex(name.to_owned(), weight);
            self.vertex_map.insert(name.to_owned(), id);
        }
        self.vertex_map.get(name).unwrap().clone()
    }

    fn add_rule(&mut self, name: &str, body: &RuleBody) {
        let mut next = VecDeque::new();
        next.push_back((name.to_owned(), body.clone()));
        while let Some((next_name, next_body)) = next.pop_front() {
            let weight = if next_body.is_terminal() { 0 } else { 1 };
            let uid = self.get_or_insert_vertex(&next_name, weight);
            let (new_body, sub_exps) = next_body.hoist_subexpressions(name, &mut self.gen);
            for (fresh_name, sub_exp) in sub_exps {
                next.push_back((fresh_name.to_owned(), sub_exp.clone()));
            }
            for sym_name in new_body.get_nonterminals() {
                if sym_name != name {
                    let weight = if new_body.is_terminal() { 0 } else { 1 };
                    let vid = self.get_or_insert_vertex(&sym_name, weight);
                    self.graph.add_edge(uid, vid);
                }
            }
            self.rules.insert(next_name, new_body);
        }
    }

    fn gen(&self) -> Vec<AstType> {
        let order = graph::topo_sort(&self.graph);
        let mut bodies = Vec::new();
        for s in order {
            bodies.push((s, self.rules.get(s).unwrap()));
        }
        bodies
            .iter()
            .map(|(name, body)| AstType::from_rule(name, body))
            .collect()
    }
}

fn main() {
    let s = fs::read_to_string("tests/arithmetic/grammar.json").unwrap();
    let g: Grammar = serde_json::from_str(&s).unwrap();
    let mut ty_gen = TypeGenerator::new();
    let rules: Vec<(&String, &RuleBody)> = g.rules.iter().collect();
    for (name, body) in rules {
        ty_gen.add_rule(name, body);
    }
    for t in ty_gen.gen() {
        println!("type {}", t);
    }
}
