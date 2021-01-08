use crate::dot::DotString;
use std::borrow::Cow;

/// Controls how an image is positioned within its containing node.
/// Only has an effect when the image is smaller than the containing node.
///
/// The default is to be centered both horizontally and vertically.
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

/// Controls how an image fills its containing node.
/// In general, the image is given its natural size, (cf. dpi), and the node size is made large
/// enough to contain its image, its label, its margin, and its peripheries.
///
/// Its width and height will also be at least as large as its minimum width and height.
/// If, however, fixedsize=true, the width and height attributes specify the exact size of the node.
///
/// During rendering, in the default case (imagescale=false), the image retains its natural size.
///
/// If imagescale=true, the image is uniformly scaled (i.e., its aspect ratio is preserved) to fit
/// inside the node.
/// At least one dimension of the image will be as large as possible given the size of the node.
///
/// When imagescale=width, the width of the image is scaled to fill the node width.
///
/// When imagescale=both, both the height and the width are scaled separately to fill the node.
///
/// As with the case of expansion, if imagescale=true, width and height are scaled uniformly.
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
