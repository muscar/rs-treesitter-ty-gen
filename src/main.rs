mod ast_types;
mod grammar;
mod graph;
mod type_gen;

use crate::{grammar::Grammar, type_gen::TypeGenerator};

fn main() {
    let g = Grammar::from_file("tests/arithmetic/grammar.json");
    let mut ty_gen = TypeGenerator::new();
    for r in g.get_rules() {
        ty_gen.add_rule(&r);
    }
    let tys = ty_gen.gen();
    println!("type {}", tys[0]);
    for t in tys.iter().skip(1) {
        println!("and {}", t);
    }
    println!(";");
}
