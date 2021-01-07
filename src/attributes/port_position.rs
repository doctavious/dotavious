use crate::attributes::compass_point::CompassPoint;
use crate::dot::DotString;
use std::borrow::Cow;

/// Modifier indicating where on a node an edge should be aimed.
/// If Port is used, the corresponding node must either have record shape with one of its
/// fields having the given portname, or have an HTML-like label, one of whose components has a
/// PORT attribute set to portname.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PortPosition {
    Port {
        port_name: String,
        compass_point: Option<CompassPoint>,
    },
    Compass(CompassPoint),
}

// TODO: AsRef vs this?
// See https://github.com/Peternator7/strum/blob/96ee0a9a307ec7d1a39809fb59037bd4e11557cc/strum/src/lib.rs
impl<'a> DotString<'a> for PortPosition {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            PortPosition::Port {
                port_name,
                compass_point,
            } => {
                let mut dot_string = port_name.to_owned();
                if let Some(compass_point) = compass_point {
                    dot_string
                        .push_str(format!(":{}", compass_point.dot_string()).as_str());
                }
                dot_string.into()
            }
            PortPosition::Compass(p) => p.dot_string().into(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::attributes::{PortPosition, CompassPoint};
    use crate::DotString;

    #[test]
    fn port_dot_string() {
        assert_eq!(
            "port_0",
            PortPosition::Port {
                port_name: "port_0".to_string(),
                compass_point: None
            }.dot_string()
        );
        assert_eq!(
            "port_0:ne",
            PortPosition::Port {
                port_name: "port_0".to_string(),
                compass_point: Some(CompassPoint::NE)
            }.dot_string()
        );
    }

    #[test]
    fn compass_dot_string() {
        assert_eq!("ne", PortPosition::Compass(CompassPoint::NE).dot_string());
    }
}