use crate::dot::DotString;
use std::borrow::Cow;

pub enum ArrowType {
    Normal,
    Dot,
    Odot,
    None,
    Empty,
    Diamond,
    Ediamond,
    Box,
    Open,
    Vee,
    Inv,
    Invdot,
    Invodot,
    Tee,
    Invempty,
    Odiamond,
    Crow,
    Obox,
    Halfopen,
}

impl<'a> DotString<'a> for ArrowType {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            ArrowType::Normal => "normal".into(),
            ArrowType::Dot => "dot".into(),
            ArrowType::Odot => "odot".into(),
            ArrowType::None => "none".into(),
            ArrowType::Empty => "empty".into(),
            ArrowType::Diamond => "diamond".into(),
            ArrowType::Ediamond => "ediamond".into(),
            ArrowType::Box => "box".into(),
            ArrowType::Open => "open".into(),
            ArrowType::Vee => "vee".into(),
            ArrowType::Inv => "inv".into(),
            ArrowType::Invdot => "invdot".into(),
            ArrowType::Invodot => "invodot".into(),
            ArrowType::Tee => "tee".into(),
            ArrowType::Invempty => "invempty".into(),
            ArrowType::Odiamond => "odiamond".into(),
            ArrowType::Crow => "crow".into(),
            ArrowType::Obox => "obox".into(),
            ArrowType::Halfopen => "halfopen".into(),
        }
    }
}
