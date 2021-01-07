use crate::dot::DotString;
use std::borrow::Cow;

/// The modes "node", "clust" or "graph" specify that the components should be packed together
/// tightly, using the specified granularity.
pub enum PackMode {
    /// causes packing at the node and edge level, with no overlapping of these objects.
    /// This produces a layout with the least area, but it also allows interleaving,
    /// where a node of one component may lie between two nodes in another component.
    Node,

    /// guarantees that top-level clusters are kept intact.
    /// What effect a value has also depends on the layout algorithm.
    Cluster,

    /// does a packing using the bounding box of the component.
    /// Thus, there will be a rectangular region around a component free of elements of any other component.
    Graph,
    // TODO: array - "array(_flags)?(%d)?"
}

impl<'a> DotString<'a> for PackMode {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            PackMode::Node => "node".into(),
            PackMode::Cluster => "clust".into(),
            PackMode::Graph => "graph".into(),
        }
    }
}
