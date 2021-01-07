//! Simple graphviz dot file format output.

pub mod attributes;
pub mod dot;

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
