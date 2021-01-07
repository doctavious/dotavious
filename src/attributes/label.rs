use crate::dot::DotString;
use std::borrow::Cow;

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
