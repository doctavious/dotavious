use crate::dot::DotString;
use std::borrow::Cow;

/// Justification for cluster labels.
///
/// Right, the label is right-justified within bounding rectangle
/// Left, left-justified
/// Else the label is centered.
///
/// Note that a subgraph inherits attributes from its parent.
/// Thus, if the root graph sets labeljust=l, the subgraph inherits this value.
pub enum LabelJustification {
    Left,
    Right,
    Center,
}

impl<'a> DotString<'a> for LabelJustification {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            LabelJustification::Left => "l".into(),
            LabelJustification::Right => "r".into(),
            LabelJustification::Center => "c".into(),
        }
    }
}

/// Vertical placement of labels for nodes, root graphs and clusters.
///
/// For graphs and clusters, only labelloc=t and labelloc=b are allowed,
/// corresponding to placement at the top and bottom, respectively.
///
/// By default, root graph labels go on the bottom and cluster labels go on the top.
///
/// Note that a subgraph inherits attributes from its parent.
/// Thus, if the root graph sets labelloc=b, the subgraph inherits this value.
///
/// For nodes, this attribute is used only when the height of the node is larger than the height
/// of its label.
///
/// If labelloc=t, labelloc=c, labelloc=b, the label is aligned with the top, centered, or aligned
/// with the bottom of the node, respectively.
///
/// By default, the label is vertically centered.
pub enum LabelLocation {
    Top,
    Center,
    Bottom,
}

impl<'a> DotString<'a> for LabelLocation {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            LabelLocation::Top => "t".into(),
            LabelLocation::Center => "c".into(),
            LabelLocation::Bottom => "b".into(),
        }
    }
}
