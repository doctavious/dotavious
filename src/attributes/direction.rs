use crate::dot::DotString;
use std::borrow::Cow;

pub enum Direction {
    Forward,
    Back,
    Both,
    None,
}

impl<'a> DotString<'a> for Direction {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Direction::Forward => "forward".into(),
            Direction::Back => "back".into(),
            Direction::Both => "both".into(),
            Direction::None => "none".into(),
        }
    }
}
