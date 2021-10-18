use std::collections::{HashMap, VecDeque};

use crate::{
    ast_types::AstType,
    grammar::{Rule, RuleBody},
    graph::{self, Graph, VertexId},
    name_gen::NameGen,
};

pub struct TypeGenerator {
    graph: Graph<String>,
    vertex_map: HashMap<String, VertexId>,
    rules: HashMap<String, RuleBody>,
    extras: Vec<Rule>,
    name_gen: NameGen,
}

impl TypeGenerator {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            vertex_map: HashMap::new(),
            rules: HashMap::new(),
            extras: Vec::new(),
            name_gen: NameGen::new(),
        }
    }

    fn get_or_insert_vertex(&mut self, name: &str) -> VertexId {
        if !self.vertex_map.contains_key(name) {
            let id = self.graph.add_vertex(name.to_owned());
            self.vertex_map.insert(name.to_owned(), id);
        }
        *self.vertex_map.get(name).unwrap()
    }

    pub fn add_rule(&mut self, rule: &Rule) {
        let mut next = VecDeque::new();
        next.push_back((rule.name.to_owned(), rule.body.clone()));
        while let Some((next_name, next_body)) = next.pop_front() {
            let (new_body, sub_exps) = next_body.hoist_subexps(
                &rule.name,
                |r| matches!(r, RuleBody::Choice { .. }),
                &mut self.name_gen,
            );
            for (fresh_name, sub_exp) in sub_exps {
                next.push_back((fresh_name.to_owned(), sub_exp.clone()));
            }
            if rule.is_extra {
                self.extras.push(rule.clone());
            } else {
                self.add_to_dag(&rule.name, &next_name, &new_body.get_nonterminals())
            }
            self.rules.insert(next_name, new_body);
        }
    }

    fn add_to_dag(&mut self, rule_name: &str, name: &str, nonterminals: &[String]) {
        let uid = self.get_or_insert_vertex(name);
        for sym_name in nonterminals {
            if sym_name != rule_name {
                let vid = self.get_or_insert_vertex(sym_name);
                self.graph.add_edge(uid, vid);
            }
        }
    }

    pub fn gen(&self) -> Vec<AstType> {
        let order = graph::topo_sort(&self.graph);
        let mut bodies = Vec::new();
        for s in order {
            bodies.push((s, self.rules.get(s).unwrap()));
        }
        bodies.extend(self.extras.iter().map(|r| (&r.name, &r.body)));
        bodies
            .iter()
            .map(|(name, body)| AstType::from_rule(name, body))
            .collect()
    }
}
