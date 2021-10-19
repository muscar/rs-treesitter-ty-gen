# Design notes

## Source code organization

Besides the main module, the project is split in the following modules:

- `grammar`: Defines types for representing treesitter grammars, as well as
  utility functions to manipulate grammar rules;
- `ast_types`: Defines types for representing ReasonML types, as well as
  functions to print the ReasonML code for those types;
- `type_gen`: Implements the logic of converting treesitter grammars to
  ReasonML type hierarchies;
- `graph`: A simple implementation of directed graphs using adjacency lists,
  and an implementation of topological sorting for them; and
- `name_gen`: A simple type to generate unique names with a given prefix--a
  glorified counter.

## Simplifying grammar rules

The treesitter grammar DSL supports anonymous alternative--`choice`-- as
arguments of other builtin functions, e.g.

```
program: $ => repeat(choice(
    $.assignment_statement,
    $.expression_statement
)),
```

Translated directly this would result in an anonymous sum type in ReasonML:

```
type program = list(assignment_statement | expression_statement)
```

ReasonML's type system doesn't support anonymous sum types. To work around this
`type_gen` hoists `choice`s out of rules, and creates intermediate rules. The
previous example would (conceptually) get rewritten as:

```
program: $ => repeat(program_0),

proggram_0: choice(
    $.assignment_statement,
    $.expression_statement
),
```

And the resulting ReasonML types are:

```
type program = list(program_0)
and program_0 = 
  | PROGRAM_0_CTOR_0 (assignment_statement)
  | PROGRAM_0_CTOR_1 (expression_statement)
```

Hoisting is implemented in `hoist_subexps` and it rewrites the source rule
using `map_subexps`.

## ReasonML type declaration ordering

The `grammar` module uses [serde](https://serde.rs/) to deserialize the JSON
grammar. Unfortunately the grammar rules are deserialized as a Rust `HashMap`.
As `HashMap`s are not order preserving, the ReasonML type hierarchy would
vary from run to run.

In order to make the output consistent across runs, `type_gen` builds a DAG
of the types, and outputs them after performing a topological sorting on the
DAG. This has the nice side effect that the "most important" types come first.

### `extra` rules

The rules defined in the `extra` section of a treesitter grammar don't have to
be explicitly mentioned in the other grammar rules. This is a problem for the
above approach since their corresponding types would appear quite early in the
generated type hierarchy.

To work around this, `type_gen` treats extra rules specially. They don't get
added to the DAG, and they get printed last. This works under the assumption
that extra rules are very simple, mostly terminals.

## Things that could be improved

- The program only supports a subset of the treesitter grammar DSL;
- There is a fair amount of cloning and copying. This could probably be
  minimised with some care;
- `type_gen` is fairly specific to ReasonML. There is no reason wjy this
  program can't support multiple output languages. But this would require a
  bit of refactoring. There is a bit of coupling between `type_gen` and
  `ast_types`;
- Error handling is mostly missing.