mod graph;

use graph::Graph;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display, Write},
    fs,
};

#[derive(Serialize, Deserialize, Debug)]
struct Grammar {
    name: String,
    rules: HashMap<String, RuleBody>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Clone, Debug)]
struct AstType {
    name: Option<String>,
    repr: AstTypeRepr,
}

impl AstType {
    fn named(name: String, repr: AstTypeRepr) -> Self {
        Self {
            name: Some(name),
            repr,
        }
    }

    fn anonymous(repr: AstTypeRepr) -> Self {
        Self { name: None, repr }
    }
}

#[derive(Clone, Debug)]
enum AstTypeRepr {
    Sum { tys: Vec<AstTypeRepr> },
    Product { tys: Vec<AstTypeRepr> },
    Ctor { name: String, tys: Vec<AstTypeRepr> },
    Name(String),
}

impl AstTypeRepr {
    fn with_new_subtys(&self, new_tys: Vec<AstTypeRepr>) -> AstTypeRepr {
        match self {
            AstTypeRepr::Sum { .. } => AstTypeRepr::Sum { tys: new_tys },
            AstTypeRepr::Product { .. } => AstTypeRepr::Product { tys: new_tys },
            AstTypeRepr::Ctor { name, .. } => AstTypeRepr::Ctor {
                name: name.clone(),
                tys: new_tys,
            },
            n @ _ => n.clone(),
        }
    }

    fn get_subtypes(&self) -> Option<&Vec<AstTypeRepr>> {
        match self {
            AstTypeRepr::Sum { tys } => Some(tys),
            AstTypeRepr::Product { tys } => Some(tys),
            AstTypeRepr::Ctor { tys, .. } => Some(tys),
            AstTypeRepr::Name(_) => None,
        }
    }

    fn needs_name(&self) -> bool {
        matches!(self, AstTypeRepr::Sum { .. })
    }
}

struct NameGen {
    prefix: String,
    cnt: usize,
}

impl NameGen {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned(),
            cnt: 1,
        }
    }

    fn gen(&mut self) -> String {
        let name = format!("{}{}", self.prefix, self.cnt);
        self.cnt += 1;
        name
    }
}

fn visit(b: &RuleBody) -> AstTypeRepr {
    match b {
        RuleBody::Repeat { content } => AstTypeRepr::Ctor {
            name: "list".to_owned(),
            tys: vec![visit(&content)],
        },
        RuleBody::Choice { members } => AstTypeRepr::Sum {
            tys: members.iter().map(visit).collect(),
        },
        RuleBody::Seq { members } => AstTypeRepr::Product {
            tys: members.iter().map(|r| visit(r)).collect(),
        },
        RuleBody::PrecLeft { content } => visit(content),
        RuleBody::Symbol { name } => AstTypeRepr::Name(name.to_owned()),
        RuleBody::String { .. } => AstTypeRepr::Name("string".to_owned()),
        RuleBody::Pattern { .. } => AstTypeRepr::Name("string".to_owned()),
    }
}

fn normalize_ast_ty(
    name: String,
    n: &AstTypeRepr,
    graph: &mut Graph<String>,
    ng: &mut NameGen,
) -> Vec<AstType> {
    let mut ns = Vec::new();
    let mut next = VecDeque::new();
    next.push_back((name, n));
    while let Some((name, x)) = next.pop_front() {
        let uid = graph.add_vertex(name.clone());
        let repr = if let Some(tys) = x.get_subtypes() {
            let mut new_tys = vec![];
            for ty in tys {
                if ty.needs_name() {
                    let new_name = ng.gen();
                    next.push_back((new_name.clone(), ty));
                    new_tys.push(AstTypeRepr::Name(new_name.clone()));
                    let vid = graph.add_vertex(new_name);
                    graph.add_edge(uid, vid)
                } else {
                    new_tys.push(ty.clone());
                }
            }
            x.with_new_subtys(new_tys.clone())
        } else {
            x.clone()
        };
        ns.push(AstType::named(name, repr));
    }
    ns
}

impl Display for AstTypeRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstTypeRepr::Ctor { name, tys } => {
                f.write_str(name)?;
                if tys.len() > 0 {
                    f.write_char('(')?;
                    std::fmt::Display::fmt(&tys[0], f)?;
                    for ty in tys.iter().skip(1) {
                        f.write_char(',')?;
                        std::fmt::Display::fmt(&ty, f)?;
                    }
                    f.write_char(')')
                } else {
                    Ok(())
                }
            }
            AstTypeRepr::Sum { tys } => {
                let mut ng = NameGen::new("Ctor");
                for ty in tys {
                    f.write_fmt(format_args!("| {}(", ng.gen()))?;
                    std::fmt::Display::fmt(&ty, f)?;
                    f.write_char(')')?;
                    f.write_char('\n')?;
                }
                Ok(())
            }
            AstTypeRepr::Product { tys } => {
                f.write_char('(')?;
                std::fmt::Display::fmt(&tys[0], f)?;
                for ty in tys.iter().skip(1) {
                    f.write_char(',')?;
                    std::fmt::Display::fmt(&ty, f)?;
                }
                f.write_char(')')
            }
            AstTypeRepr::Name(name) => f.write_str(name),
        }
    }
}

fn main() {
    let s = fs::read_to_string("tests/arithmetic/grammar.json").unwrap();
    let g: Grammar = serde_json::from_str(&s).unwrap();
    println!("{:?}", g);
    let mut graph = Graph::new();
    let mut ng = NameGen::new("ty");
    let tys: Vec<AstType> = g
        .rules
        .iter()
        .flat_map(|(name, rule)| {
            normalize_ast_ty(name.to_string(), &visit(&rule), &mut graph, &mut ng)
        })
        .collect();

    for ty in tys {
        println!("type {} = {}", ty.name.unwrap(), ty.repr);
    }
}
