# Dotavious
![crates.io](https://img.shields.io/crates/v/dotavious.svg)](https://crates.io/crates/dotavious)
[![Released API docs](https://docs.rs/dotavious/badge.svg)](https://docs.rs/dotavious)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![CI](https://github.com/doctavious/dotavious/workflows/CI/badge.svg)](https://github.com/doctavious/dotavious/actions?query=workflow%3ACI)

A library for generating [Graphviz](https://graphviz.org/) [DOT](https://graphviz.org/doc/info/lang.html) language files 
for visualizing graphs.

## Constraints / Limitations

- Not every Attribute is fully documented/described. 
  However, all those which have specific allowed values should be covered. 
- Deprecated Attributes are not defined.


## Quickstart

```rust
use dotavious::{Dot, Edge, Graph, GraphBuilder, Node};
use std::io;
use std::io::Read;

// can also start building a named undirected graph via `GraphBuilder::new_named_undirected`
let graph = GraphBuilder::new_named_directed("example")
        .add_node(Node::new("N0"))
        .add_node(Node::new("N1"))
        .add_edge(Edge::new("N0", "N1"))
        .build()
        .unwrap();

let dot = Dot { graph };
println!("{}", dot);
```
which produces
```
digraph example {
    N0;
    N1;
    N0 -> N1;
}
```
and when rendered will look like

![README example rendered](readme-example.png?raw=true)
