#![feature(box_patterns)]
mod graph;

use graph::{Graph, VertexId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    fs,
};

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
        idx: &mut usize,
    ) -> (RuleBody, Vec<(String, RuleBody)>) {
        let mut subexps = Vec::new();
        let result = match self {
            RuleBody::Repeat { content } => {
                if content.is_terminal() {
                    self.clone()
                } else {
                    let fresh_name = format!("{}_{}", name, idx);
                    *idx += 1;
                    subexps.push((fresh_name.clone(), *content.clone()));
                    RuleBody::Repeat {
                        content: Box::new(RuleBody::Symbol { name: fresh_name }),
                    }
                }
            }
            RuleBody::Choice { members } => {
                let mut new_members = Vec::new();
                for b in members {
                    let new_b = if b.is_terminal() {
                        b.clone()
                    } else {
                        let fresh_name = format!("{}_{}", name, idx);
                        *idx += 1;
                        subexps.push((fresh_name.clone(), b.clone()));
                        RuleBody::Symbol { name: fresh_name }
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
                    let new_b = if b.is_terminal() {
                        b.clone()
                    } else {
                        let fresh_name = format!("{}_{}", name, idx);
                        *idx += 1;
                        subexps.push((fresh_name.clone(), b.clone()));
                        RuleBody::Symbol { name: fresh_name }
                    };
                    new_members.push(new_b);
                }
                RuleBody::Seq {
                    members: new_members,
                }
            }
            RuleBody::PrecLeft { content } => *content.clone(),
            RuleBody::Symbol { .. } | RuleBody::String { .. } | RuleBody::Pattern { .. } => {
                self.clone()
            }
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

struct TypeGenerator {
    graph: Graph<String>,
    vertex_map: HashMap<String, VertexId>,
    rules: HashMap<String, RuleBody>,
    fresh_rule_idx: usize,
}

impl TypeGenerator {
    fn new() -> Self {
        Self {
            graph: Graph::new(),
            vertex_map: HashMap::new(),
            rules: HashMap::new(),
            fresh_rule_idx: 0,
        }
    }

    fn get_or_insert_vertex(&mut self, name: &str) -> VertexId {
        if !self.vertex_map.contains_key(name) {
            let id = self.graph.add_vertex(name.to_owned());
            self.vertex_map.insert(name.to_owned(), id);
        }
        self.vertex_map.get(name).unwrap().clone()
    }

    fn add_rule(&mut self, name: &str, body: &RuleBody) {
        let mut next = VecDeque::new();
        next.push_back((name.to_owned(), body.clone()));
        while let Some((next_name, next_body)) = next.pop_front() {
            let uid = self.get_or_insert_vertex(&next_name);
            let (new_body, sub_exps) =
                next_body.hoist_subexpressions(name, &mut self.fresh_rule_idx);
            for (fresh_name, sub_exp) in sub_exps {
                next.push_back((fresh_name.to_owned(), sub_exp.clone()));
                let vid = self.get_or_insert_vertex(&fresh_name);
                self.graph.add_edge(uid, vid);
            }
            for sym_name in new_body.get_nonterminals() {
                if sym_name != name {
                    let vid = self.get_or_insert_vertex(&sym_name);
                    self.graph.add_edge(uid, vid);
                }
            }
            self.rules.insert(next_name, new_body);
        }
    }

    fn print_type(&self, body: &RuleBody) {
        match body {
            RuleBody::Repeat { content } => {
                print!("list(");
                self.print_type(content);
                print!(")");
            }
            RuleBody::Choice { members } => {
                for b in members {
                    print!("\n| ");
                    self.print_type(b);
                }
            }
            RuleBody::Seq { members } => {
                let mut it = members.iter();
                self.print_type(it.next().unwrap());
                for b in it.skip(1) {
                    print!(", ");
                    self.print_type(b);
                }
            }
            RuleBody::PrecLeft { content } => self.print_type(content),
            RuleBody::Symbol { name } => print!("{}", name),
            RuleBody::String { .. } | RuleBody::Pattern { .. } => print!("string"),
        }
    }

    fn gen(&self) {
        // for v in self.graph.vertices() {
        //     println!(
        //         "{:?} -> {:?}",
        //         v.value,
        //         self.graph
        //             .get_out_edges(v.id)
        //             .iter()
        //             .map(|u| self.graph.get_vertex(*u).value)
        //             .collect::<Vec<&String>>()
        //     );
        // }

        let order = graph::topo_sort(&self.graph);
        let mut bodies = Vec::new();
        for s in order {
            bodies.push((s, self.rules.get(s).unwrap()));
        }
        let mut prefix = "type";
        for (s, b) in bodies {
            print!("{} {} = ", prefix, s);
            self.print_type(b);
            println!();
            prefix = "and";
        }
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
    for (name, body) in &ty_gen.rules {
        println!("{} -> {:?}", name, body);
    }
    ty_gen.gen();
}
