#![doc(html_root_url = "https://docs.rs/dotavious/0.1.0")]
#![cfg_attr(docsrs, deny(broken_intra_doc_links))]

//! Dotavious provides bindings to generate [DOT](https://graphviz.org/doc/info/lang.html)
//! code used by the Graphviz (http://graphviz.org/) for visualising graphs.
//! It also provides strongly typed attribute functions and offers almost complete
//! coverage of all Graphviz attributes and syntax.
//!
//! # Examples
//!
//! First example provides a basic directed graph:
//! 2 nodes connected by a single edge
//!
//! ```rust
//! use dotavious::{Dot, Edge, Graph, GraphBuilder, Node};
//!
//! let g = GraphBuilder::new_directed(Some("example".to_string()))
//!         .add_node(Node::new("N0".to_string()))
//!         .add_node(Node::new("N1".to_string()))
//!         .add_edge(Edge::new("N0".to_string(), "N1".to_string()))
//!         .build()
//!         .unwrap();
//!
//! let dot = Dot { graph: g };
//! println!("{}", dot);
//! ```
//! Produces
//! ```dot
//! digraph example {
//!     N0;
//!     N1;
//!     N0 -> N1;
//! }
//! ```
//!
//! We can also output to a Writer via the `render` function
//! ```rust
//! use dotavious::{Dot, Edge, Graph, GraphBuilder, Node};
//! use std::io;
//! use std::io::Read;
//!
//! let g = GraphBuilder::new_directed(Some("example".to_string()))
//!         .add_node(Node::new("N0".to_string()))
//!         .add_node(Node::new("N1".to_string()))
//!         .add_edge(Edge::new("N0".to_string(), "N1".to_string()))
//!         .build()
//!         .unwrap();
//!
//! let dot = Dot { graph: g };
//! let mut writer= Vec::new();
//! dot.render(&mut writer).unwrap();
//!
//! // output to graphviz DOT formatted string
//! let mut dot_string = String::new();
//! Read::read_to_string(&mut &*writer, &mut dot_string).unwrap();
//! println!("{}", dot_string);
//! ```
//!
//! Second example provides a more complex graph showcasing
//! Dotavious' various builders and strongly typed attribute
//! functions.
//!
//! ```rust
//! use dotavious::attributes::{
//!     AttributeText, Color, CompassPoint, EdgeAttributes, EdgeStyle,
//!     GraphAttributeStatementBuilder, GraphAttributes, GraphStyle, NodeAttributes,
//!     NodeStyle, PortPosition, RankDir, Shape,
//! };
//! use dotavious::{
//!     Dot, Edge, EdgeAttributeStatementBuilder, EdgeBuilder, Graph,
//!     GraphBuilder, Node, NodeAttributeStatementBuilder, NodeBuilder,
//!     SubGraphBuilder
//! };
//! use std::io;
//! use std::io::Read;
//!
//! let cluster_0 = SubGraphBuilder::new(Some("cluster_0".to_string()))
//!     .add_graph_attributes(
//!         GraphAttributeStatementBuilder::new()
//!             .label("process #1".to_string())
//!             .style(GraphStyle::Filled)
//!             .color(Color::Named("lightgrey"))
//!             .build()
//!             .unwrap(),
//!     )
//!     .add_node_attributes(
//!         NodeAttributeStatementBuilder::new()
//!             .style(NodeStyle::Filled)
//!             .color(Color::Named("white"))
//!             .build()
//!             .unwrap(),
//!     )
//!     .add_edge(Edge::new("a0".to_string(), "a1".to_string()))
//!     .add_edge(Edge::new("a1".to_string(), "a2".to_string()))
//!     .add_edge(Edge::new("a2".to_string(), "a3".to_string()))
//!     .build()
//!     .unwrap();
//!
//! let cluster_1 = SubGraphBuilder::new(Some("cluster_1".to_string()))
//!     .add_graph_attributes(
//!         GraphAttributeStatementBuilder::new()
//!             .label("process #2".to_string())
//!             .style(GraphStyle::Filled)
//!             .color(Color::Named("blue"))
//!             .build()
//!             .unwrap(),
//!     )
//!     .add_node_attributes(
//!         NodeAttributeStatementBuilder::new()
//!             .style(NodeStyle::Filled)
//!             .build()
//!             .unwrap(),
//!     )
//!     .add_edge(Edge::new("b0".to_string(), "b1".to_string()))
//!     .add_edge(Edge::new("b1".to_string(), "b2".to_string()))
//!     .add_edge(Edge::new("b2".to_string(), "b3".to_string()))
//!     .build()
//!     .unwrap();
//!
//! let g = GraphBuilder::new_directed(Some("G".to_string()))
//!     .add_node(
//!         NodeBuilder::new("start".to_string())
//!             .shape(Shape::Mdiamond)
//!             .build()
//!             .unwrap(),
//!     )
//!     .add_node(
//!         NodeBuilder::new("end".to_string())
//!             .shape(Shape::Msquare)
//!             .build()
//!             .unwrap(),
//!     )
//!     .add_sub_graph(cluster_0)
//!     .add_sub_graph(cluster_1)
//!     .add_edge(Edge::new("start".to_string(), "a0".to_string()))
//!     .add_edge(Edge::new("start".to_string(), "b0".to_string()))
//!     .add_edge(Edge::new("a1".to_string(), "b3".to_string()))
//!     .add_edge(Edge::new("b2".to_string(), "a3".to_string()))
//!     .add_edge(Edge::new("a3".to_string(), "a0".to_string()))
//!     .add_edge(Edge::new("a3".to_string(), "end".to_string()))
//!     .add_edge(Edge::new("b3".to_string(), "end".to_string()))
//!     .build();
//! ```
//!
//! Produces
//! ```dot
//! digraph G {
//!     subgraph cluster_0 {
//!         graph [label="process #1", style=filled, color="lightgrey"];
//!         node [style=filled, color="white"];
//!         a0 -> a1;
//!         a1 -> a2;
//!         a2 -> a3;
//!     }
//!
//!     subgraph cluster_1 {
//!         graph [label="process #2", style=filled, color="blue"];
//!         node [style=filled];
//!         b0 -> b1;
//!         b1 -> b2;
//!         b2 -> b3;
//!     }
//!
//!     start [shape=Mdiamond];
//!     end [shape=Msquare];
//!     start -> a0;
//!     start -> b0;
//!     a1 -> b3;
//!     b2 -> a3;
//!     a3 -> a0;
//!     a3 -> end;
//!     b3 -> end;
//! }
//! ```

pub mod attributes;
pub mod dot;
pub mod validation;

#[doc(hidden)]
pub use crate::dot::{
    Dot, DotString, Edge, EdgeAttributeStatementBuilder, EdgeBuilder, Graph,
    GraphBuilder, Node, NodeAttributeStatementBuilder, NodeBuilder, SubGraphBuilder,
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
