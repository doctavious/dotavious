use crate::dot::DotString;
use std::borrow::Cow;

/// Sets the aspect ratio (drawing height/drawing width) for the drawing.
/// Note that this is adjusted before the size attribute constraints are enforced.
/// In addition, the calculations usually ignore the node sizes, so the final drawing size may only
/// approximate what is desired.
pub enum Ratio {
    /// Ratio is taken as the desired aspect ratio.
    /// Then, if the actual aspect ratio is less than the desired ratio, the drawing height is
    /// scaled up to achieve the desired ratio; if the actual ratio is greater than that desired
    /// ratio, the drawing width is scaled up.
    Aspect(f32),
    /// If fill and the size attribute is set, node positions are scaled, separately in both x and y,
    /// so that the final drawing exactly fills the specified size.
    /// If both size values exceed the width and height of the drawing, then both coordinate values
    /// of each node are scaled up accordingly. However, if either size dimension is smaller than
    /// the corresponding dimension in the drawing, one dimension is scaled up so that the
    /// final drawing has the same aspect ratio as specified by size.
    /// Then, when rendered, the layout will be scaled down uniformly in both dimensions to fit
    /// the given size, which may cause nodes and text to shrink as well. This may not be what
    /// the user wants, but it avoids the hard problem of how to reposition the nodes in an
    /// acceptable fashion to reduce the drawing size.
    Fill,
    /// If compress and the size attribute is set, dot attempts to compress the initial
    /// layout to fit in the given size. This achieves a tighter packing of nodes but reduces
    /// the balance and symmetry.
    Compress,
    /// the size attribute is set, and both the width and the height of the graph are less than
    /// the value in size, node positions are scaled uniformly until at least one dimension
    /// fits size exactly.
    /// Note that this is distinct from using size as the desired size, as here the drawing is
    /// expanded before edges are generated and all node and text sizes remain unchanged.
    Expand,
    /// the page attribute is set and the graph cannot be drawn on a single page,
    /// then size is set to an “ideal” value.
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
