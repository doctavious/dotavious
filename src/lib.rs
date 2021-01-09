#![doc(html_root_url = "https://docs.rs/dotavious/0.1.0")]
#![cfg_attr(docsrs, deny(broken_intra_doc_links))]

//! Dotavious provides bindings to generate [DOT](https://graphviz.org/doc/info/lang.html)
//! code used by the Graphviz (http://graphviz.org/) for visualising graphs.
//!
//! Main features of the graphviz library include:
//! * Almost complete coverage of all Graphviz attributes and syntax.
//!
//! # Example
//!
//! ```rust
//! use dotavious::attributes::{AttributeText, GraphAttributeStatementBuilder, GraphAttributes};
//! use dotavious::{
//!     Dot, Edge, EdgeAttributeStatementBuilder, EdgeAttributes, EdgeBuilder, Graph,
//!     GraphBuilder, Node, NodeAttributeStatementBuilder, NodeAttributes, NodeBuilder,
//! };
//! use std::io;
//! use std::io::Read;
//!
//! let g = GraphBuilder::new_directed(Some("example".to_string()))
//!         .add_node(Node::new("N0".to_string()))
//!         .add_node(Node::new("N1".to_string()))
//!         .add_edge(Edge::new("N0".to_string(), "N1".to_string()))
//!         .build();
//!
//! let mut writer= Vec::new();
//! let dot = Dot { graph: g };
//! dot.render(&mut writer).unwrap();
//!
//! // output to graphviz DOT formatted string
//! let mut dot_string = String::new();
//! Read::read_to_string(&mut &*writer, &mut dot_string).unwrap();
//! println!("{}", dot_string);
//! ```
//! Produces
//! ```dot
//! digraph example {
//!     N0;
//!     N1;
//!     N0 -> N1;
//! }
//! ```


pub mod attributes;
pub mod dot;

#[doc(hidden)]
pub use crate::dot::{
    Dot, DotString, Edge, EdgeAttributeStatementBuilder, EdgeAttributes, EdgeBuilder,
    Graph, GraphBuilder, Node, NodeAttributeStatementBuilder, NodeAttributes,
    NodeBuilder,
};

// TODO: support adding edge based on index of nodes?
// TODO: handle render options
// TODO: explicit attribute methods with type safety and enforce constraints
// i'm thinking we have NodeTraits/GraphTraits/EdgeTraits (what about none? is that a graph trait?)
// which will have default methods that use an associated type field called "state" or "attributes" etc
// TODO: implement Clone for Graph
// TODO: see if we can get any insights from Haskell implementation
// https://hackage.haskell.org/package/graphviz-2999.20.1.0/docs/Data-GraphViz-Attributes-Complete.html#t:Point
// - I like this: A summary of known current constraints/limitations/differences:
// Add a DPoint enum?
// /// Either a Double or a (2D) Point (i.e. created with Point::new_2d).
// pub enum DPoint {
//     Double(f32),
//     Point(Point),
// }
