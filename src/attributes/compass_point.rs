use crate::dot::DotString;
use std::borrow::Cow;

// TODO: not sure we need this enum but should support setting nodeport either via
// headport / tailport attributes e.g. a -> b [tailport=se]
// or via edge declaration using the syntax node name:port_name e.g. a -> b:se
// aka compass
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CompassPoint {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    /// pecifies the center of the node or port
    C,
    /// specifies that an appropriate side of the port adjacent to the exterior of the node should
    /// be used, if such exists.
    None,
}

impl<'a> DotString<'a> for CompassPoint {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            CompassPoint::N => "n".into(),
            CompassPoint::NE => "ne".into(),
            CompassPoint::E => "e".into(),
            CompassPoint::SE => "se".into(),
            CompassPoint::S => "s".into(),
            CompassPoint::SW => "sw".into(),
            CompassPoint::W => "w".into(),
            CompassPoint::NW => "nw".into(),
            CompassPoint::C => "c".into(),
            CompassPoint::None => "_".into(),
        }
    }
}
