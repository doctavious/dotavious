use crate::dot::DotString;
use std::borrow::Cow;

/// These specify the 8 row or column major orders for traversing a rectangular array,
/// the first character corresponding to the major order and the second to the minor order.
/// Thus, for “BL”, the major order is from bottom to top, and the minor order is from left to right.
/// This means the bottom row is traversed first, from left to right, then the next row up,
/// from left to right, and so on, until the topmost row is traversed
pub enum PageDirection {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
    RightBottom,
    RightTop,
    LeftBottom,
    LeftTop,
}

impl<'a> DotString<'a> for PageDirection {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            PageDirection::BottomLeft => "BL".into(),
            PageDirection::BottomRight => "BR".into(),
            PageDirection::TopLeft => "TL".into(),
            PageDirection::TopRight => "TR".into(),
            PageDirection::RightBottom => "RB".into(),
            PageDirection::RightTop => "RT".into(),
            PageDirection::LeftBottom => "LB".into(),
            PageDirection::LeftTop => "LT".into(),
        }
    }
}
