# r2c-thc

Generate ReasonML type definitions for a subset of treesitter grammars.

# Build & run

You'll need a Rust stable toolchain to build the project. You can use
[rustup](https://rustup.rs/) to install one.

To build, simply clone the project and (in the project directory) run:

```
cargo build --release
```

Or, to run it directly:

```
cargo run --release -- <path>
```

E.g.

```
cargo run --release -- tests/arithmetic/grammar.json
```

The output should match:

```
type program = list(program_0)
and program_0 =
 | PROGRAM_0_CTOR_0 (assignment_statement)
 | PROGRAM_0_CTOR_1 (expression_statement)
and assignment_statement = (variable, string, expression, string)
and expression_statement = (expression, string)
and expression =
 | EXPRESSION_CTOR_0 (variable)
 | EXPRESSION_CTOR_1 (number)
 | EXPRESSION_CTOR_2 ((expression, string, expression))
 | EXPRESSION_CTOR_3 ((expression, string, expression))
 | EXPRESSION_CTOR_4 ((expression, string, expression))
 | EXPRESSION_CTOR_5 ((expression, string, expression))
 | EXPRESSION_CTOR_6 ((expression, string, expression))
and variable = string
and number = string
and comment = string
;
```