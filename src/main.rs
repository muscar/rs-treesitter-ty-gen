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
        match self {
            RuleBody::Symbol { .. } | RuleBody::String { .. } | RuleBody::Pattern { .. } => true,
            _ => false,
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

    fn fresh_rule_name(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.fresh_rule_idx);
        self.fresh_rule_idx += 1;
        name
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
        next.push_back((name.to_owned(), body));
        while let Some((next_name, next_body)) = next.pop_front() {
            match next_body {
                RuleBody::Repeat { content } => {
                    let uid = self.get_or_insert_vertex(&next_name);
                    let new_content = if !content.is_terminal() {
                        let fresh_name = self.fresh_rule_name(&next_name);
                        next.push_back((fresh_name.to_owned(), content));
                        let vid = self.get_or_insert_vertex(&fresh_name);
                        self.graph.add_edge(uid, vid);
                        Box::new(RuleBody::Symbol { name: fresh_name })
                    } else {
                        match **content {
                            RuleBody::Symbol { name: ref sym_name } if *sym_name != name => {
                                let vid = self.get_or_insert_vertex(&sym_name);
                                self.graph.add_edge(uid, vid);
                            }
                            _ => (),
                        }
                        content.clone()
                    };
                    self.rules.insert(
                        next_name,
                        RuleBody::Repeat {
                            content: new_content,
                        },
                    );
                }
                RuleBody::Choice { members } => {
                    let uid = self.get_or_insert_vertex(&next_name);
                    let mut new_members = Vec::new();
                    for b in members {
                        let new_b = if !b.is_terminal() {
                            let fresh_name = self.fresh_rule_name(&next_name);
                            next.push_back((fresh_name.to_owned(), b));
                            let vid = self.get_or_insert_vertex(&fresh_name);
                            self.graph.add_edge(uid, vid);
                            RuleBody::Symbol { name: fresh_name }
                        } else {
                            match b {
                                RuleBody::Symbol { name: sym_name } if *sym_name != name => {
                                    let vid = self.get_or_insert_vertex(&sym_name);
                                    self.graph.add_edge(uid, vid);
                                }
                                _ => (),
                            }
                            b.clone()
                        };
                        new_members.push(new_b);
                    }
                    self.rules.insert(
                        next_name,
                        RuleBody::Choice {
                            members: new_members,
                        },
                    );
                }
                RuleBody::Seq { members } => {
                    let uid = self.get_or_insert_vertex(&next_name);
                    let mut new_members = Vec::new();
                    for b in members {
                        let new_b = if !b.is_terminal() {
                            let fresh_name = self.fresh_rule_name(&next_name);
                            next.push_back((fresh_name.to_owned(), b));
                            let vid = self.get_or_insert_vertex(&fresh_name);
                            self.graph.add_edge(uid, vid);
                            RuleBody::Symbol { name: fresh_name }
                        } else {
                            match b {
                                RuleBody::Symbol { name: sym_name } if *sym_name != name => {
                                    let vid = self.get_or_insert_vertex(&sym_name);
                                    self.graph.add_edge(uid, vid);
                                }
                                _ => (),
                            }
                            b.clone()
                        };
                        new_members.push(new_b);
                    }
                    self.rules.insert(
                        next_name,
                        RuleBody::Seq {
                            members: new_members,
                        },
                    );
                }
                RuleBody::PrecLeft { content } => {
                    next.push_back((next_name, content));
                }
                _ => {
                    self.get_or_insert_vertex(&next_name);
                    self.rules.insert(next_name, next_body.clone());
                }
            }
        }
    }

    fn add_extra(&mut self, body: &RuleBody) {
        let uid = self.get_or_insert_vertex("$extras");
        match body {
            RuleBody::Symbol { name } => {
                let vid = self.get_or_insert_vertex(name);
                self.graph.add_edge(uid, vid);
            }
            _ => (),
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
        for v in self.graph.vertices() {
            println!(
                "{:?} -> {:?}",
                v.value,
                self.graph
                    .get_out_edges(v.id)
                    .iter()
                    .map(|u| self.graph.get_vertex(*u).value)
                    .collect::<Vec<&String>>()
            );
        }

        let order = graph::topo_sort(&self.graph);
        let mut prefix = "type";
        for s in order.iter() {
            if *s != "$extras" {
                print!("{} {} = ", prefix, s);
                self.print_type(self.rules.get(*s).unwrap());
                println!();
                prefix = "and";
            }
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
    for body in g.extras {
        ty_gen.add_extra(&body);
    }
    for (name, body) in &ty_gen.rules {
        println!("{} -> {:?}", name, body);
    }
    ty_gen.gen();
}
