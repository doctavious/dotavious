use crate::dot::DotString;
use std::borrow::Cow;

pub enum Ordering {
    In,
    Out,
}

impl<'a> DotString<'a> for Ordering {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Ordering::In => "in".into(),
            Ordering::Out => "out".into(),
        }
    }
}
