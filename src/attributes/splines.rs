use crate::dot::DotString;
use std::borrow::Cow;

/// Spline, edges are drawn as splines routed around nodes
/// Line, edges are drawn as line segments
/// Polygon, specifies that edges should be drawn as polylines.
/// Ortho, specifies edges should be routed as polylines of axis-aligned segments.
/// Curved, specifies edges should be drawn as curved arcs.
/// splines=line and splines=spline can be used as synonyms for
/// splines=false and splines=true, respectively.
pub enum Splines {
    Line,
    Spline,
    None,
    Curved,
    Polyline,
    Ortho,
}

impl<'a> DotString<'a> for Splines {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Splines::Line => "line".into(),
            Splines::Spline => "spline".into(),
            Splines::None => "none".into(),
            Splines::Curved => "curved".into(),
            Splines::Polyline => "polyline".into(),
            Splines::Ortho => "ortho".into(),
        }
    }
}
