use crate::dot::DotString;
use std::borrow::Cow;

/// Corresponding to directed graphs drawn from top to bottom, from left to right,
/// from bottom to top, and from right to left, respectively.
pub enum RankDir {
    TopBottom,
    LeftRight,
    BottomTop,
    RightLeft,
}

impl<'a> DotString<'a> for RankDir {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            RankDir::TopBottom => "TB".into(),
            RankDir::LeftRight => "LR".into(),
            RankDir::BottomTop => "BT".into(),
            RankDir::RightLeft => "RL".into(),
        }
    }
}
