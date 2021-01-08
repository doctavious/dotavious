use crate::dot::DotString;
use std::borrow::Cow;

/// If out, then the outedges of a node, that is, edges with the node as its tail node,
/// must appear left-to-right in the same order in which they are defined in the input.
///
/// If in, then the inedges of a node must appear left-to-right in the same order in which they are
/// defined in the input.
///
/// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph
/// or subgraph.
///
/// Note that the graph attribute takes precedence over the node attribute.
pub enum Ordering {
    In,
    Out,
}

impl<'a> DotString<'a> for Ordering {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Ordering::In => "in".into(),
            Ordering::Out => "out".into(),
        }
    }
}
