use std::collections::{HashMap, VecDeque};

use crate::{
    ast_types::AstType,
    grammar::{NameGen, Rule, RuleBody},
    graph::{self, Graph, VertexId},
};

pub struct TypeGenerator {
    graph: Graph<String, bool>,
    vertex_map: HashMap<String, VertexId>,
    rules: HashMap<String, RuleBody>,
    name_gen: NameGen,
}

impl TypeGenerator {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            vertex_map: HashMap::new(),
            rules: HashMap::new(),
            name_gen: NameGen::new(),
        }
    }

    fn get_or_insert_vertex(&mut self, name: &str, weight: bool) -> VertexId {
        if !self.vertex_map.contains_key(name) {
            let id = self.graph.add_vertex(name.to_owned(), weight);
            self.vertex_map.insert(name.to_owned(), id);
        }
        *self.vertex_map.get(name).unwrap()
    }

    pub fn add_rule(&mut self, rule: &Rule) {
        let mut next = VecDeque::new();
        next.push_back((rule.name.to_owned(), rule.body.clone()));
        while let Some((next_name, next_body)) = next.pop_front() {
            let uid = self.get_or_insert_vertex(&next_name, !next_body.is_terminal());
            let (new_body, sub_exps) = next_body.hoist_subexps(
                &rule.name,
                |r| matches!(r, RuleBody::Choice { .. }),
                &mut self.name_gen,
            );
            for (fresh_name, sub_exp) in sub_exps {
                next.push_back((fresh_name.to_owned(), sub_exp.clone()));
            }
            for sym_name in new_body.get_nonterminals() {
                if sym_name != rule.name {
                    let vid = self.get_or_insert_vertex(&sym_name, !next_body.is_terminal());
                    self.graph.add_edge(uid, vid);
                }
            }
            self.rules.insert(next_name, new_body);
        }
    }

    pub fn gen(&self) -> Vec<AstType> {
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
