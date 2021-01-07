use crate::dot::DotString;
use std::borrow::Cow;

pub enum Ratio {
    Aspect(f32),
    Fill,
    Compress,
    Expand,
    Auto,
}

impl<'a> DotString<'a> for Ratio {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Ratio::Aspect(aspect) => aspect.to_string().into(),
            Ratio::Fill => "fill".into(),
            Ratio::Compress => "compress".into(),
            Ratio::Expand => "expand".into(),
            Ratio::Auto => "auto".into(),
        }
    }
}
