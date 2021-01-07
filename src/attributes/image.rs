use crate::dot::DotString;
use std::borrow::Cow;

pub enum ImagePosition {
    TopLeft,
    TopCentered,
    TopRight,
    MiddleLeft,
    MiddleCentered,
    MiddleRight,
    BottomLeft,
    BottomCentered,
    BottomRight,
}

impl<'a> DotString<'a> for ImagePosition {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            ImagePosition::TopLeft => "tl".into(),
            ImagePosition::TopCentered => "tc".into(),
            ImagePosition::TopRight => "tr".into(),
            ImagePosition::MiddleLeft => "ml".into(),
            ImagePosition::MiddleCentered => "mc".into(),
            ImagePosition::MiddleRight => "mr".into(),
            ImagePosition::BottomLeft => "bl".into(),
            ImagePosition::BottomCentered => "bc".into(),
            ImagePosition::BottomRight => "br".into(),
        }
    }
}

pub enum ImageScale {
    Width,
    Height,
    Both,
}

impl<'a> DotString<'a> for ImageScale {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            ImageScale::Width => "width".into(),
            ImageScale::Height => "height".into(),
            ImageScale::Both => "both".into(),
        }
    }
}
