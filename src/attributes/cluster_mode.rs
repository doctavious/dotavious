use crate::dot::DotString;
use std::borrow::Cow;

pub enum ClusterMode {
    Local,
    Global,
    None,
}

impl<'a> DotString<'a> for ClusterMode {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            ClusterMode::Local => "local".into(),
            ClusterMode::Global => "global".into(),
            ClusterMode::None => "none".into(),
        }
    }
}
