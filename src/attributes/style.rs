use crate::dot::DotString;
use std::borrow::Cow;

pub enum NodeStyle {
    Bold,
    Dashed,
    Diagonals,
    Dotted,
    Filled,
    Invisible,
    Rounded,
    Solid,
    Stripped,
    Radical,
    Wedged,
}

impl<'a> DotString<'a> for NodeStyle {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            NodeStyle::Bold => "bold".into(),
            NodeStyle::Dashed => "dashed".into(),
            NodeStyle::Diagonals => "diagonals".into(),
            NodeStyle::Dotted => "dotted".into(),
            NodeStyle::Filled => "filled".into(),
            NodeStyle::Invisible => "invisible".into(),
            NodeStyle::Rounded => "rounded".into(),
            NodeStyle::Solid => "solid".into(),
            NodeStyle::Stripped => "stripped".into(),
            NodeStyle::Radical => "radical".into(),
            NodeStyle::Wedged => "wedged".into(),
        }
    }
}

pub enum EdgeStyle {
    Bold,
    Dashed,
    Dotted,
    Invisible,
    Solid,
    Tapered,
}

impl<'a> DotString<'a> for EdgeStyle {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            EdgeStyle::Bold => "bold".into(),
            EdgeStyle::Dashed => "dashed".into(),
            EdgeStyle::Dotted => "dotted".into(),
            EdgeStyle::Invisible => "invisible".into(),
            EdgeStyle::Solid => "solid".into(),
            EdgeStyle::Tapered => "tapered".into(),
        }
    }
}

pub enum GraphStyle {
    Filled,
    Radical,
    Rounded,
    Striped,
}

impl<'a> DotString<'a> for GraphStyle {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            GraphStyle::Filled => "filled".into(),
            GraphStyle::Radical => "radical".into(),
            GraphStyle::Rounded => "rounded".into(),
            GraphStyle::Striped => "striped".into(),
        }
    }
}

// TODO: this might be a bit much to in order to avoid some duplication
// probably not worth it but is pattern is cool nonetheless
pub enum Styles {
    Edge(EdgeStyle),
    Node(NodeStyle),
    Graph(GraphStyle),
}

impl<'a> DotString<'a> for Styles {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Styles::Edge(s) => s.dot_string(),
            Styles::Node(s) => s.dot_string(),
            Styles::Graph(s) => s.dot_string(),
        }
    }
}
