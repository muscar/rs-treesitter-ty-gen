use std::env;

mod ast_types;
mod grammar;
mod graph;
mod name_gen;
mod type_gen;

use crate::{grammar::Grammar, type_gen::TypeGenerator};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("usage: {} <path>", args[0]);
    }
    let g = Grammar::from_file(&args[1]);
    let mut ty_gen = TypeGenerator::new();
    for r in g.get_rules() {
        ty_gen.add_rule(r);
    }
    let tys = ty_gen.gen();
    ast_types::print_type_hierarchy(&tys);
}
