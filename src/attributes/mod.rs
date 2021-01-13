// TODO: docs

mod arrow_type;
mod cluster_mode;
mod color;
mod compass_point;
mod direction;
mod image;
mod label;
mod ordering;
mod output_mode;
mod pack_mode;
mod page_direction;
mod point;
mod port_position;
mod rankdir;
mod ratio;
mod rectangle;
mod shape;
mod spline_type;
mod splines;
mod style;

pub use crate::attributes::arrow_type::ArrowType;
pub use crate::attributes::cluster_mode::ClusterMode;
pub use crate::attributes::color::{Color, ColorList, IntoWeightedColor, WeightedColor};
pub use crate::attributes::compass_point::CompassPoint;
pub use crate::attributes::direction::Direction;
pub use crate::attributes::image::{ImagePosition, ImageScale};
pub use crate::attributes::label::{LabelJustification, LabelLocation};
pub use crate::attributes::ordering::Ordering;
pub use crate::attributes::output_mode::OutputMode;
pub use crate::attributes::pack_mode::PackMode;
pub use crate::attributes::page_direction::PageDirection;
pub use crate::attributes::point::Point;
pub use crate::attributes::port_position::PortPosition;
pub use crate::attributes::rankdir::RankDir;
pub use crate::attributes::ratio::Ratio;
pub use crate::attributes::rectangle::Rectangle;
pub use crate::attributes::shape::Shape;
pub use crate::attributes::spline_type::SplineType;
pub use crate::attributes::splines::Splines;
pub use crate::attributes::style::{EdgeStyle, GraphStyle, NodeStyle, Styles};
#[doc(hidden)]
pub use crate::attributes::AttributeText::{AttrStr, EscStr, HtmlStr, QuotedStr};
use crate::dot::DotString;
use indexmap::map::IndexMap;
use std::borrow::Cow;
use std::collections::HashMap;

/// The text for a graphviz label on a node or edge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AttributeText<'a> {
    /// Preserves the text directly as is.
    AttrStr(Cow<'a, str>),

    /// This kind of label uses the graphviz label escString type:
    /// <http://www.graphviz.org/doc/info/attrs.html#k:escString>
    ///
    /// Occurrences of backslashes (`\`) are not escaped; instead they
    /// are interpreted as initiating an escString escape sequence.
    ///
    /// Escape sequences of particular interest: in addition to `\n`
    /// to break a line (centering the line preceding the `\n`), there
    /// are also the escape sequences `\l` which left-justifies the
    /// preceding line and `\r` which right-justifies it.
    EscStr(Cow<'a, str>),

    /// This uses a graphviz [HTML string label][html].
    /// The string is printed exactly as given, but between `<` and `>`.
    /// **No escaping is performed.**
    ///
    /// [html]: https://graphviz.org/doc/info/shapes.html#html
    HtmlStr(Cow<'a, str>),

    /// Preserves the text directly as is but wrapped in quotes.
    ///
    /// Occurrences of backslashes (`\`) are escaped, and thus appear
    /// as backslashes in the rendered label.
    QuotedStr(Cow<'a, str>),
}

impl<'a> AttributeText<'a> {
    pub fn attr<S: Into<Cow<'a, str>>>(s: S) -> AttributeText<'a> {
        AttrStr(s.into())
    }

    pub fn escaped<S: Into<Cow<'a, str>>>(s: S) -> AttributeText<'a> {
        EscStr(s.into())
    }

    pub fn html<S: Into<Cow<'a, str>>>(s: S) -> AttributeText<'a> {
        HtmlStr(s.into())
    }

    pub fn quoted<S: Into<Cow<'a, str>>>(s: S) -> AttributeText<'a> {
        QuotedStr(s.into())
    }

    fn escape_char<F>(c: char, mut f: F)
    where
        F: FnMut(char),
    {
        match c {
            // not escaping \\, since Graphviz escString needs to
            // interpret backslashes; see EscStr above.
            '\\' => f(c),
            _ => {
                for c in c.escape_default() {
                    f(c)
                }
            }
        }
    }

    fn escape_str(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            AttributeText::escape_char(c, |c| out.push(c));
        }
        out
    }

    /// Renders text as string suitable for a attribute in a .dot file.
    /// This includes quotes or suitable delimiters.
    pub fn dot_string(&self) -> String {
        match *self {
            AttrStr(ref s) => format!("{}", s),
            EscStr(ref s) => format!("\"{}\"", AttributeText::escape_str(&s)),
            HtmlStr(ref s) => format!("<{}>", s),
            QuotedStr(ref s) => format!("\"{}\"", s.escape_default()),
        }
    }
}

impl<'a> From<ArrowType> for AttributeText<'a> {
    fn from(arrow_type: ArrowType) -> Self {
        AttributeText::attr(arrow_type.dot_string())
    }
}

impl<'a> From<bool> for AttributeText<'a> {
    fn from(v: bool) -> Self {
        AttributeText::attr(v.to_string())
    }
}

impl<'a> From<ClusterMode> for AttributeText<'a> {
    fn from(mode: ClusterMode) -> Self {
        AttributeText::quoted(mode.dot_string())
    }
}

impl<'a> From<Color<'a>> for AttributeText<'a> {
    fn from(color: Color<'a>) -> Self {
        AttributeText::quoted(color.dot_string())
    }
}

impl<'a> From<ColorList<'a>> for AttributeText<'a> {
    fn from(color_list: ColorList<'a>) -> Self {
        AttributeText::quoted(color_list.dot_string())
    }
}

impl<'a> From<CompassPoint> for AttributeText<'a> {
    fn from(compass: CompassPoint) -> Self {
        AttributeText::quoted(compass.dot_string())
    }
}

impl<'a> From<Direction> for AttributeText<'a> {
    fn from(direction: Direction) -> Self {
        AttributeText::attr(direction.dot_string())
    }
}

impl<'a> From<EdgeStyle> for AttributeText<'a> {
    fn from(style: EdgeStyle) -> Self {
        AttributeText::attr(style.dot_string())
    }
}

impl<'a> From<f32> for AttributeText<'a> {
    fn from(v: f32) -> Self {
        AttributeText::attr(v.to_string())
    }
}

impl<'a> From<GraphStyle> for AttributeText<'a> {
    fn from(style: GraphStyle) -> Self {
        AttributeText::attr(style.dot_string())
    }
}

impl<'a> From<ImagePosition> for AttributeText<'a> {
    fn from(pos: ImagePosition) -> Self {
        AttributeText::quoted(pos.dot_string())
    }
}

impl<'a> From<ImageScale> for AttributeText<'a> {
    fn from(scale: ImageScale) -> Self {
        AttributeText::quoted(scale.dot_string())
    }
}

impl<'a> From<LabelJustification> for AttributeText<'a> {
    fn from(label_justification: LabelJustification) -> Self {
        AttributeText::attr(label_justification.dot_string())
    }
}

impl<'a> From<LabelLocation> for AttributeText<'a> {
    fn from(label_location: LabelLocation) -> Self {
        AttributeText::attr(label_location.dot_string())
    }
}

impl<'a> From<NodeStyle> for AttributeText<'a> {
    fn from(style: NodeStyle) -> Self {
        AttributeText::attr(style.dot_string())
    }
}

impl<'a> From<Ordering> for AttributeText<'a> {
    fn from(ordering: Ordering) -> Self {
        AttributeText::quoted(ordering.dot_string())
    }
}

impl<'a> From<OutputMode> for AttributeText<'a> {
    fn from(mode: OutputMode) -> Self {
        AttributeText::quoted(mode.dot_string())
    }
}

impl<'a> From<PackMode> for AttributeText<'a> {
    fn from(mode: PackMode) -> Self {
        AttributeText::quoted(mode.dot_string())
    }
}

impl<'a> From<PageDirection> for AttributeText<'a> {
    fn from(page_direction: PageDirection) -> Self {
        AttributeText::attr(page_direction.dot_string())
    }
}

impl<'a> From<Point> for AttributeText<'a> {
    fn from(point: Point) -> Self {
        AttributeText::quoted(point.dot_string())
    }
}

impl<'a> From<PortPosition> for AttributeText<'a> {
    fn from(port_position: PortPosition) -> Self {
        AttributeText::quoted(port_position.dot_string())
    }
}

impl<'a> From<RankDir> for AttributeText<'a> {
    fn from(rank_dir: RankDir) -> Self {
        AttributeText::attr(rank_dir.dot_string())
    }
}

impl<'a> From<Ratio> for AttributeText<'a> {
    fn from(ratio: Ratio) -> Self {
        match ratio {
            Ratio::Aspect(_aspect) => AttributeText::attr(ratio.dot_string()),
            _ => AttributeText::quoted(ratio.dot_string()),
        }
    }
}

impl<'a> From<Rectangle> for AttributeText<'a> {
    fn from(rectangle: Rectangle) -> Self {
        AttributeText::quoted(rectangle.dot_string())
    }
}

impl<'a> From<Shape> for AttributeText<'a> {
    fn from(shape: Shape) -> Self {
        AttributeText::attr(shape.dot_string())
    }
}

impl<'a> From<Splines> for AttributeText<'a> {
    fn from(splines: Splines) -> Self {
        AttributeText::quoted(splines.dot_string())
    }
}

impl<'a> From<SplineType> for AttributeText<'a> {
    fn from(spline_type: SplineType) -> Self {
        AttributeText::quoted(spline_type.dot_string())
    }
}

impl<'a> From<Styles> for AttributeText<'a> {
    fn from(styles: Styles) -> Self {
        match styles {
            Styles::Edge(s) => AttributeText::from(s),
            Styles::Node(s) => AttributeText::from(s),
            Styles::Graph(s) => AttributeText::from(s),
        }
    }
}

impl<'a> From<u32> for AttributeText<'a> {
    fn from(v: u32) -> Self {
        AttributeText::attr(v.to_string())
    }
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Clone)]
pub enum AttributeType {
    Graph,
    Node,
    Edge,
}

pub trait AttributeStatement<'a> {
    fn get_attribute_statement_type(&self) -> &'static str;

    fn get_attributes(&self) -> &IndexMap<String, AttributeText<'a>>;

    fn dot_string(&self) -> String {
        if self.get_attributes().is_empty() {
            return String::from("");
        }

        format!(
            "{}{};",
            self.get_attribute_statement_type(),
            fmt_attributes(self.get_attributes())
        )
    }
}

pub trait GraphAttributes<'a> {
    fn background(&mut self, background: String) -> &mut Self {
        self.add_attribute("_background", AttributeText::attr(background))
    }

    /// The color used as the background for entire canvas.
    fn background_color(&mut self, background_color: Color<'a>) -> &mut Self {
        self.add_attribute("bgcolor", AttributeText::from(background_color))
    }

    // TODO: constrain
    /// The color used as the background for entire canvas with a gradient fill.
    /// A colon-separated list of weighted color values: WC(:WC)* where each WC has the form C(;F)?
    /// with C a color value and the optional F a floating-point number, 0 ≤ F ≤ 1.
    /// The sum of the floating-point numbers in a colorList must sum to at most 1.
    fn background_colorlist(&mut self, background_colors: ColorList<'a>) -> &mut Self {
        self.add_attribute("bgcolor", AttributeText::from(background_colors))
    }

    /// Type: rect which is "%f,%f,%f,%f"
    /// The rectangle llx,lly,urx,ury gives the coordinates, in points, of the lower-left corner (llx,lly)
    /// and the upper-right corner (urx,ury).
    fn bounding_box(&mut self, bounding_box: String) -> &mut Self {
        self.add_attribute("bb", AttributeText::quoted(bounding_box))
    }

    /// If true, the drawing is centered in the output canvas.
    fn center(&mut self, center: bool) -> &mut Self {
        self.add_attribute("center", AttributeText::from(center))
    }

    /// Specifies the character encoding used when interpreting string input as a text label.
    fn charset(&mut self, charset: String) -> &mut Self {
        self.add_attribute("charset", AttributeText::quoted(charset))
    }

    /// Classnames to attach to the node, edge, graph, or cluster’s SVG element.
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    fn class(&mut self, class: String) -> &mut Self {
        Attributes::class(self.get_attributes_mut(), class);
        self
    }

    /// Mode used for handling clusters.
    /// If clusterrank=local, a subgraph whose name begins with cluster is given special treatment.
    /// The subgraph is laid out separately, and then integrated as a unit into its parent graph,
    ///  with a bounding rectangle drawn about it.
    /// If the cluster has a label parameter, this label is displayed within the rectangle.
    /// Note also that there can be clusters within clusters.
    /// The modes clusterrank=global and clusterrank=none appear to be identical, both turning off the special cluster processing.
    fn cluster_rank(&mut self, cluster_rank: ClusterMode) -> &mut Self {
        self.add_attribute("clusterrank", AttributeText::from(cluster_rank))
    }

    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    /// If any fraction is used, the colors are drawn in series, with each color being given
    /// roughly its specified fraction of the edge.
    fn color(&mut self, color: Color<'a>) -> &mut Self {
        Attributes::color(self.get_attributes_mut(), color);
        self
    }

    /// This attribute specifies a color scheme namespace: the context for interpreting color names.
    /// In particular, if a color value has form "xxx" or "//xxx", then the color xxx will be evaluated
    /// according to the current color scheme. If no color scheme is set, the standard X11 naming is used.
    /// For example, if colorscheme=bugn9, then color=7 is interpreted as color="/bugn9/7".
    fn color_scheme(&mut self, color_scheme: String) -> &mut Self {
        Attributes::color_scheme(self.get_attributes_mut(), color_scheme);
        self
    }

    /// Comments are inserted into output. Device-dependent
    fn comment(&mut self, comment: String) -> &mut Self {
        Attributes::comment(self.get_attributes_mut(), comment);
        self
    }

    fn compound(&mut self, compound: String) -> &mut Self {
        self.add_attribute("compound", AttributeText::quoted(compound))
    }

    fn concentrate(&mut self, concentrate: String) -> &mut Self {
        self.add_attribute("concentrate", AttributeText::quoted(concentrate))
    }

    /// Specifies the expected number of pixels per inch on a display device.
    /// Also known as resolution
    fn dpi(&mut self, dpi: f32) -> &mut Self {
        self.add_attribute("dpi", AttributeText::from(dpi))
    }

    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    fn fill_color(&mut self, fill_color: Color<'a>) -> &mut Self {
        Attributes::fill_color(self.get_attributes_mut(), fill_color);
        self
    }

    /// Color used to fill the background, with a gradient, of a node or cluster assuming
    /// style=filled, or a filled arrowhead.
    fn fill_color_with_colorlist(&mut self, fill_colors: ColorList<'a>) -> &mut Self {
        Attributes::fill_color_with_colorlist(self.get_attributes_mut(), fill_colors);
        self
    }

    /// Color used to fill the background, with a gradient, of a node or cluster assuming
    /// style=filled, or a filled arrowhead.
    /// TODO: example
    /// [crate::attributes::GraphAttributes::dpi]
    fn fill_color_with_iter<I>(&mut self, fill_colors: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: IntoWeightedColor<'a>,
    {
        Attributes::fill_color_with_iter(self.get_attributes_mut(), fill_colors);
        self
    }

    /// Color used for text.
    fn font_color(&mut self, font_color: Color<'a>) -> &mut Self {
        Attributes::font_color(self.get_attributes_mut(), font_color);
        self
    }

    /// Font used for text.
    fn font_name(&mut self, font_name: String) -> &mut Self {
        Attributes::font_name(self.get_attributes_mut(), font_name);
        self
    }

    fn font_names(&mut self, font_names: String) -> &mut Self {
        self.add_attribute("fontnames", AttributeText::quoted(font_names))
    }

    fn font_path(&mut self, font_path: String) -> &mut Self {
        self.add_attribute("fontpath", AttributeText::quoted(font_path))
    }

    // TODO: constrain
    /// Font size, in points, used for text.
    /// default: 14.0, minimum: 1.0
    fn font_size(&mut self, font_size: f32) -> &mut Self {
        Attributes::font_size(self.get_attributes_mut(), font_size);
        self
    }

    fn force_label(&mut self, force_label: bool) -> &mut Self {
        self.add_attribute("forcelabel", AttributeText::from(force_label))
    }

    /// If a gradient fill is being used, this determines the angle of the fill.
    fn gradient_angle(&mut self, gradient_angle: u32) -> &mut Self {
        Attributes::gradient_angle(self.get_attributes_mut(), gradient_angle);
        self
    }

    fn image_path(&mut self, image_path: String) -> &mut Self {
        self.add_attribute("imagepath", AttributeText::escaped(image_path))
    }

    /// An escString or an HTML label.
    fn label(&mut self, label: String) -> &mut Self {
        Attributes::label(self.get_attributes_mut(), label);
        self
    }

    /// If labeljust=r, the label is right-justified within bounding rectangle
    /// If labeljust=l, left-justified
    /// Else the label is centered.
    fn label_justification(
        &mut self,
        label_justification: LabelJustification,
    ) -> &mut Self {
        self.add_attribute("labeljust", AttributeText::from(label_justification))
    }

    // Vertical placement of labels for nodes, root graphs and clusters.
    // For graphs and clusters, only labelloc=t and labelloc=b are allowed, corresponding to
    // placement at the top and bottom, respectively.
    // By default, root graph labels go on the bottom and cluster labels go on the top.
    // Note that a subgraph inherits attributes from its parent. Thus, if the root graph sets
    // labelloc=b, the subgraph inherits this value.
    // For nodes, this attribute is used only when the height of the node is larger than the height
    // of its label.
    // If labelloc=t, labelloc=c, labelloc=b, the label is aligned with the top, centered, or
    // aligned with the bottom of the node, respectively.
    // By default, the label is vertically centered.
    fn label_location(&mut self, label_location: LabelLocation) -> &mut Self {
        Attributes::label_location(self.get_attributes_mut(), label_location);
        self
    }

    fn landscape(&mut self, landscape: bool) -> &mut Self {
        self.add_attribute("landscape", AttributeText::from(landscape))
    }

    /// Specifies the separator characters used to split an attribute of type layerRange into a
    /// list of ranges.
    fn layer_list_sep(&mut self, layer_list_sep: String) -> &mut Self {
        self.add_attribute("layerlistsep", AttributeText::attr(layer_list_sep))
    }

    /// Specifies a linearly ordered list of layer names attached to the graph
    /// The graph is then output in separate layers.
    /// Only those components belonging to the current output layer appear.
    fn layers(&mut self, layers: String) -> &mut Self {
        Attributes::layer(self.get_attributes_mut(), layers);
        self
    }

    /// Selects a list of layers to be emitted.
    fn layer_select(&mut self, layer_select: String) -> &mut Self {
        self.add_attribute("layerselect", AttributeText::attr(layer_select))
    }

    /// Specifies the separator characters used to split the layers attribute into a list of layer names.
    /// default: ":\t "
    fn layer_sep(&mut self, layer_sep: String) -> &mut Self {
        self.add_attribute("layersep", AttributeText::attr(layer_sep))
    }

    /// Height of graph or cluster label, in inches.
    fn lheight(&mut self, lheight: f32) -> &mut Self {
        self.add_attribute("lheight", AttributeText::from(lheight))
    }

    /// Label position
    /// The position indicates the center of the label.
    fn label_position(&mut self, lp: Point) -> &mut Self {
        Attributes::label_position(self.get_attributes_mut(), lp);
        self
    }

    /// Width of graph or cluster label, in inches.
    fn lwidth(&mut self, lwidth: f32) -> &mut Self {
        self.add_attribute("lwidth", AttributeText::from(lwidth))
    }

    /// Sets x and y margins of canvas, in inches.
    /// Both margins are set equal to the given value.
    /// See [`crate::attributes::GraphAttributes::margin_point`]
    fn margin(&mut self, margin: f32) -> &mut Self {
        self.margin_point(Point::new_2d(margin, margin))
    }

    /// Sets x and y margins of canvas, in inches.
    /// Note that the margin is not part of the drawing but just empty space left around the drawing.
    /// The margin basically corresponds to a translation of drawing, as would be necessary to
    /// center a drawing on a page. Nothing is actually drawn in the margin.
    /// To actually extend the background of a drawing, see the pad attribute.
    /// Whilst it is possible to create a Point value with either a third co-ordinate
    /// or a forced position, these are ignored for printing.
    /// By default, the value is 0.11,0.055.
    fn margin_point(&mut self, margin: Point) -> &mut Self {
        Attributes::margin(self.get_attributes_mut(), margin);
        self
    }

    /// Multiplicative scale factor used to alter the MinQuit (default = 8) and
    /// MaxIter (default = 24) parameters used during crossing minimization.
    /// These correspond to the number of tries without improvement before quitting and the
    /// maximum number of iterations in each pass.
    fn mclimit(&mut self, mclimit: f32) -> &mut Self {
        self.add_attribute("mclimit", AttributeText::from(mclimit))
    }

    /// Specifies the minimum separation between all nodes.
    fn mindist(&mut self, mindist: u32) -> &mut Self {
        self.add_attribute("mindist", AttributeText::from(mindist))
    }

    /// Whether to use a single global ranking, ignoring clusters.
    /// The original ranking algorithm in dot is recursive on clusters.
    /// This can produce fewer ranks and a more compact layout, but sometimes at the cost of a
    /// head node being place on a higher rank than the tail node.
    /// It also assumes that a node is not constrained in separate, incompatible subgraphs.
    /// For example, a node cannot be in a cluster and also be constrained by rank=same with
    /// a node not in the cluster.
    /// This allows nodes to be subject to multiple constraints.
    /// Rank constraints will usually take precedence over edge constraints.
    fn newrank(&mut self, newrank: bool) -> &mut Self {
        self.add_attribute("newrank", AttributeText::from(newrank))
    }

    // TODO: add constraint
    /// specifies the minimum space between two adjacent nodes in the same rank, in inches.
    /// default: 0.25, minimum: 0.02
    fn nodesep(&mut self, nodesep: f32) -> &mut Self {
        self.add_attribute("nodesep", AttributeText::from(nodesep))
    }

    /// By default, the justification of multi-line labels is done within the largest context that makes sense.
    /// Thus, in the label of a polygonal node, a left-justified line will align with the left side
    /// of the node (shifted by the prescribed margin).
    /// In record nodes, left-justified line will line up with the left side of the enclosing column
    /// of fields.
    /// If nojustify=true, multi-line labels will be justified in the context of itself.
    /// For example, if nojustify is set, the first label line is long, and the second is shorter
    /// and left-justified,
    /// the second will align with the left-most character in the first line, regardless of how
    /// large the node might be.
    fn no_justify(&mut self, no_justify: bool) -> &mut Self {
        Attributes::no_justify(self.get_attributes_mut(), no_justify);
        self
    }

    /// Sets number of iterations in network simplex applications.
    /// nslimit is used in computing node x coordinates.
    /// If defined, # iterations = nslimit * # nodes; otherwise, # iterations = MAXINT.
    fn nslimit(&mut self, nslimit: f32) -> &mut Self {
        self.add_attribute("nslimit", AttributeText::from(nslimit))
    }

    /// If ordering="out", then the outedges of a node, that is, edges with the node as its tail
    /// node, must appear left-to-right in the same order in which they are defined in the input.
    ///
    /// If ordering="in", then the inedges of a node must appear left-to-right in the same order in
    /// which they are defined in the input.
    ///
    /// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph
    /// or subgraph.
    ///
    /// Note that the graph attribute takes precedence over the node attribute.
    fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        Attributes::ordering(self.get_attributes_mut(), ordering);
        self
    }

    // TODO: constrain to 0 - 360. Docs say min is 360 which should be max right?
    /// Used only if rotate is not defined.
    /// Default: 0.0 and minimum: 360.0
    fn orientation(&mut self, orientation: f32) -> &mut Self {
        Attributes::orientation(self.get_attributes_mut(), orientation);
        self
    }

    /// Specify order in which nodes and edges are drawn.
    /// default: breadthfirst
    fn output_order(&mut self, output_order: OutputMode) -> &mut Self {
        self.add_attribute("outputorder", AttributeText::from(output_order))
    }

    /// Whether each connected component of the graph should be laid out separately, and then the
    /// graphs packed together.
    /// If false, the entire graph is laid out together.
    /// The granularity and method of packing is influenced by the packmode attribute.
    fn pack(&mut self, pack: bool) -> &mut Self {
        self.add_attribute("pack", AttributeText::from(pack))
    }

    // TODO: constrain to non-negative integer.
    /// Whether each connected component of the graph should be laid out separately, and then
    /// the graphs packed together.
    /// This is used as the size, in points,of a margin around each part; otherwise, a default
    /// margin of 8 is used.
    /// pack is treated as true if the value of pack iso a non-negative integer.
    fn pack_int(&mut self, pack: u32) -> &mut Self {
        self.add_attribute("pack", AttributeText::from(pack))
    }

    /// This indicates how connected components should be packed (cf. packMode).
    /// Note that defining packmode will automatically turn on packing as though one had set pack=true.
    fn pack_mode(&mut self, pack_mode: PackMode) -> &mut Self {
        self.add_attribute("packmode", AttributeText::from(pack_mode))
    }

    /// Specifies how much, in inches, to extend the drawing area around the minimal area needed
    /// to draw the graph.
    /// Both the x and y pad values are set equal to the given value.
    /// See [`crate::attributes::GraphAttributes::pad_point`]
    fn pad(&mut self, pad: f32) -> &mut Self {
        self.pad_point(Point::new_2d(pad, pad))
    }

    /// Specifies how much, in inches, to extend the drawing area around the minimal area needed to
    /// draw the graph.
    /// This area is part of the drawing and will be filled with the background color, if appropriate.
    /// default: 0.0555
    fn pad_point(&mut self, pad: Point) -> &mut Self {
        self.add_attribute("pad", AttributeText::from(pad))
    }

    /// Width and height of output pages, in inches.
    /// Value given is used for both the width and height.
    fn page(&mut self, page: f32) -> &mut Self {
        self.add_attribute("page", AttributeText::from(page))
    }

    /// Width and height of output pages, in inches.
    fn page_point(&mut self, page: Point) -> &mut Self {
        self.add_attribute("page", AttributeText::from(page))
    }

    /// The order in which pages are emitted.
    /// Used only if page is set and applicable.
    /// Limited to one of the 8 row or column major orders.
    fn page_dir(&mut self, page_dir: PageDirection) -> &mut Self {
        self.add_attribute("pagedir", AttributeText::from(page_dir))
    }

    // TODO: constrain
    /// If quantum > 0.0, node label dimensions will be rounded to integral multiples of the quantum.
    /// default: 0.0, minimum: 0.0
    fn quantum(&mut self, quantum: f32) -> &mut Self {
        self.add_attribute("quantum", AttributeText::from(quantum))
    }

    /// Sets direction of graph layout.
    /// For example, if rankdir="LR", and barring cycles, an edge T -> H; will go from left to right.
    /// By default, graphs are laid out from top to bottom.
    /// This attribute also has a side-effect in determining how record nodes are interpreted.
    /// See record shapes.
    fn rank_dir(&mut self, rank_dir: RankDir) -> &mut Self {
        self.add_attribute("rankdir", AttributeText::from(rank_dir))
    }

    /// sets the desired rank separation, in inches.
    /// This is the minimum vertical distance between the bottom of the nodes in one rank
    /// and the tops of nodes in the next. If the value contains equally,
    /// the centers of all ranks are spaced equally apart.
    /// Note that both settings are possible, e.g., ranksep="1.2 equally".
    fn rank_sep(&mut self, rank_sep: String) -> &mut Self {
        self.add_attribute("ranksep", AttributeText::attr(rank_sep))
    }

    /// Sets the aspect ratio (drawing height/drawing width) for the drawing.
    /// Note that this is adjusted before the size attribute constraints are enforced.
    fn ratio(&mut self, ratio: Ratio) -> &mut Self {
        self.add_attribute("ratio", AttributeText::from(ratio))
    }

    /// If true and there are multiple clusters, run crossing minimization a second time.
    fn remincross(&mut self, remincross: bool) -> &mut Self {
        self.add_attribute("remincross", AttributeText::from(remincross))
    }

    /// If rotate=90, sets drawing orientation to landscape.
    fn rotate(&mut self, rotate: u32) -> &mut Self {
        self.add_attribute("rotate", AttributeText::from(rotate))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at
    /// the end if showboxes=2.
    /// (Debugging, TB mode only!)
    /// default: 0, minimum: 0
    fn show_boxes(&mut self, show_boxes: u32) -> &mut Self {
        Attributes::show_boxes(self.get_attributes_mut(), show_boxes);
        self
    }

    /// Maximum width and height of drawing, in inches.
    /// Value used for both the width and the height.
    /// If defined and the drawing is larger than the given size, the drawing
    /// is uniformly scaled down so that it fits within the given size.
    /// If desired_min is true, and both both dimensions of the drawing
    /// are less than size, the drawing is scaled up uniformly until at
    /// least one dimension equals its dimension in size.
    /// See [`crate::attributes::GraphAttributes::size_point`]
    fn size(&mut self, size: u32, desired_min: bool) -> &mut Self {
        self.size_point(Point {
            x: size as f32,
            y: size as f32,
            z: None,
            force_pos: desired_min,
        })
    }

    /// Maximum width and height of drawing, in inches.
    /// If defined and the drawing is larger than the given size, the drawing
    /// is uniformly scaled down so that it fits within the given size.
    /// If desired_min is true, and both both dimensions of the drawing
    /// are less than size, the drawing is scaled up uniformly until at
    /// least one dimension equals its dimension in size.
    fn size_point(&mut self, size: Point) -> &mut Self {
        self.add_attribute("size", AttributeText::from(size))
    }

    /// If packmode indicates an array packing, sortv specifies an insertion order
    /// among the components, with smaller values inserted first.
    /// default: 0, minimum: 0
    fn sortv(&mut self, sortv: u32) -> &mut Self {
        Attributes::sortv(self.get_attributes_mut(), sortv);
        self
    }

    /// Controls how, and if, edges are represented.
    fn splines(&mut self, splines: Splines) -> &mut Self {
        self.add_attribute("splines", AttributeText::from(splines))
    }

    /// Set style information for components of the graph.
    fn style(&mut self, style: GraphStyle) -> &mut Self {
        Attributes::style(self.get_attributes_mut(), Styles::Graph(style));
        self
    }

    /// A URL or pathname specifying an XML style sheet, used in SVG output.
    /// Combine with class to style elements using CSS selectors.
    fn stylesheet(&mut self, stylesheet: String) -> &mut Self {
        self.add_attribute("stylesheet", AttributeText::attr(stylesheet))
    }

    /// If the object has a URL, this attribute determines which window of the browser is used for the URL.
    fn target(&mut self, target: String) -> &mut Self {
        Attributes::target(self.get_attributes_mut(), target);
        self
    }

    /// Whether internal bitmap rendering relies on a truecolor color model or uses a color palette.
    /// If truecolor is unset, truecolor is not used unless there is a shapefile property
    /// for some node in the graph.
    /// The output model will use the input model when possible.
    fn true_color(&mut self, true_color: bool) -> &mut Self {
        self.add_attribute("truecolor", AttributeText::from(true_color))
    }

    /// Hyperlinks incorporated into device-dependent output.
    fn url(&mut self, url: String) -> &mut Self {
        Attributes::url(self.get_attributes_mut(), url);
        self
    }

    // TODO: add a ViewPort Struct?
    /// Clipping window on final drawing.
    /// viewport supersedes any size attribute.
    /// The width and height of the viewport specify precisely the final size of the output.
    /// The viewPort W,H,Z,x,y or W,H,Z,N specifies a viewport for the final image.
    /// The pair (W,H) gives the dimensions (width and height) of the final image, in points.
    /// The optional Z is the zoom factor, i.e., the image in the original layout will be
    /// W/Z by H/Z points in size. By default, Z is 1.
    /// The optional last part is either a pair (x,y) giving a position in the original layout
    /// of the graph,
    /// in points, of the center of the viewport, or the name N of a node whose center should used
    /// as the focus.
    fn viewport(&mut self, viewport: String) -> &mut Self {
        self.add_attribute("viewport", AttributeText::attr(viewport))
    }

    /// Add an attribute to the node.
    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self;

    /// Add multiple attributes to the node.
    fn add_attributes(
        &'a mut self,
        attributes: HashMap<String, AttributeText<'a>>,
    ) -> &mut Self;

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>>;
}

impl<'a> GraphAttributes<'a> for GraphAttributeStatementBuilder<'a> {
    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attributes to the node.
    fn add_attributes(
        &'a mut self,
        attributes: HashMap<String, AttributeText<'a>>,
    ) -> &mut Self {
        self.attributes.extend(attributes);
        self
    }

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>> {
        &mut self.attributes
    }
}

// I'm not a huge fan of needing this builder but having a hard time getting around &mut without it
pub struct GraphAttributeStatementBuilder<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> GraphAttributeStatementBuilder<'a> {
    pub fn new() -> Self {
        Self {
            attributes: IndexMap::new(),
        }
    }

    pub fn build(&self) -> GraphAttributeStatement<'a> {
        GraphAttributeStatement {
            attributes: self.attributes.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GraphAttributeStatement<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> GraphAttributeStatement<'a> {
    pub fn new() -> Self {
        Self {
            attributes: IndexMap::new(),
        }
    }

    pub fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

impl<'a> AttributeStatement<'a> for GraphAttributeStatement<'a> {
    fn get_attribute_statement_type(&self) -> &'static str {
        "graph"
    }

    fn get_attributes(&self) -> &IndexMap<String, AttributeText<'a>> {
        &self.attributes
    }
}

pub(crate) struct Attributes;
impl Attributes {
    pub fn class(attributes: &mut IndexMap<String, AttributeText>, class: String) {
        Self::add_attribute(attributes, "class", AttributeText::quoted(class))
    }

    pub fn color<'a>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        color: Color<'a>,
    ) {
        Self::add_attribute(attributes, "color", AttributeText::from(color))
    }

    pub fn color_with_colorlist<'a>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        color: ColorList<'a>,
    ) {
        Self::add_attribute(attributes, "color", AttributeText::from(color))
    }

    pub fn color_scheme(
        attributes: &mut IndexMap<String, AttributeText>,
        color_scheme: String,
    ) {
        Self::add_attribute(
            attributes,
            "colorscheme",
            AttributeText::quoted(color_scheme),
        )
    }

    pub fn comment(attributes: &mut IndexMap<String, AttributeText>, comment: String) {
        Self::add_attribute(attributes, "comment", AttributeText::quoted(comment))
    }

    pub fn fill_color<'a>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        fill_color: Color<'a>,
    ) {
        Self::add_attribute(attributes, "fillcolor", AttributeText::from(fill_color))
    }

    pub fn fill_color_with_colorlist<'a>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        fill_colors: ColorList<'a>,
    ) {
        Self::add_attribute(attributes, "fillcolor", AttributeText::from(fill_colors))
    }

    pub fn fill_color_with_iter<'a, I>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        fill_colors: I,
    ) where
        I: IntoIterator,
        I::Item: IntoWeightedColor<'a>,
    {
        let colors: Vec<WeightedColor> = fill_colors
            .into_iter()
            .map(|e| e.into_weighted_color())
            .collect();

        let color_list = ColorList { colors };

        Self::add_attribute(attributes, "fillcolor", AttributeText::from(color_list))
    }

    pub fn font_color<'a>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        font_color: Color<'a>,
    ) {
        Self::add_attribute(attributes, "fontcolor", AttributeText::from(font_color))
    }

    pub fn font_name(
        attributes: &mut IndexMap<String, AttributeText>,
        font_name: String,
    ) {
        Self::add_attribute(attributes, "fontname", AttributeText::quoted(font_name))
    }

    pub fn font_size(attributes: &mut IndexMap<String, AttributeText>, font_size: f32) {
        Self::add_attribute(attributes, "fontsize", AttributeText::from(font_size))
    }

    pub fn gradient_angle(
        attributes: &mut IndexMap<String, AttributeText>,
        gradient_angle: u32,
    ) {
        Self::add_attribute(
            attributes,
            "gradientangle",
            AttributeText::from(gradient_angle),
        )
    }

    pub fn label(attributes: &mut IndexMap<String, AttributeText>, text: String) {
        Self::add_attribute(attributes, "label", AttributeText::quoted(text));
    }

    pub fn label_location(
        attributes: &mut IndexMap<String, AttributeText>,
        label_location: LabelLocation,
    ) {
        Self::add_attribute(attributes, "labelloc", AttributeText::from(label_location))
    }

    // TODO: layer struct
    pub fn layer(attributes: &mut IndexMap<String, AttributeText>, layer: String) {
        Self::add_attribute(attributes, "layer", AttributeText::attr(layer))
    }

    pub fn label_position(attributes: &mut IndexMap<String, AttributeText>, lp: Point) {
        Self::add_attribute(attributes, "lp", AttributeText::from(lp))
    }

    pub fn margin(attributes: &mut IndexMap<String, AttributeText>, margin: Point) {
        Self::add_attribute(attributes, "margin", AttributeText::from(margin))
    }

    pub fn no_justify(
        attributes: &mut IndexMap<String, AttributeText>,
        no_justify: bool,
    ) {
        Self::add_attribute(attributes, "nojustify", AttributeText::from(no_justify))
    }

    pub fn ordering(
        attributes: &mut IndexMap<String, AttributeText>,
        ordering: Ordering,
    ) {
        Self::add_attribute(attributes, "ordering", AttributeText::from(ordering))
    }

    pub fn orientation(
        attributes: &mut IndexMap<String, AttributeText>,
        orientation: f32,
    ) {
        Self::add_attribute(attributes, "orientation", AttributeText::from(orientation))
    }

    pub fn pen_width(attributes: &mut IndexMap<String, AttributeText>, pen_width: f32) {
        Self::add_attribute(attributes, "penwidth", AttributeText::from(pen_width))
    }

    // TODO: splinetype
    pub fn pos(attributes: &mut IndexMap<String, AttributeText>, pos: Point) {
        Self::add_attribute(attributes, "pos", AttributeText::from(pos))
    }

    pub fn show_boxes(
        attributes: &mut IndexMap<String, AttributeText>,
        show_boxes: u32,
    ) {
        Self::add_attribute(attributes, "showboxes", AttributeText::from(show_boxes))
    }

    pub fn sortv(attributes: &mut IndexMap<String, AttributeText>, sortv: u32) {
        Self::add_attribute(attributes, "sortv", AttributeText::from(sortv))
    }

    pub fn style(attributes: &mut IndexMap<String, AttributeText>, style: Styles) {
        Self::add_attribute(attributes, "style", AttributeText::from(style))
    }

    pub fn target(attributes: &mut IndexMap<String, AttributeText>, target: String) {
        Self::add_attribute(attributes, "target", AttributeText::escaped(target))
    }

    pub fn tooltip(attributes: &mut IndexMap<String, AttributeText>, tooltip: String) {
        Self::add_attribute(attributes, "tooltip", AttributeText::escaped(tooltip))
    }

    pub fn url(attributes: &mut IndexMap<String, AttributeText>, url: String) {
        Self::add_attribute(attributes, "url", AttributeText::escaped(url))
    }

    pub fn xlabel(attributes: &mut IndexMap<String, AttributeText>, width: String) {
        Self::add_attribute(attributes, "xlabel", AttributeText::escaped(width))
    }

    pub fn xlp(attributes: &mut IndexMap<String, AttributeText>, xlp: Point) {
        Self::add_attribute(attributes, "xlp", AttributeText::from(xlp))
    }

    pub fn add_attribute<'a, S: Into<String>>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        key: S,
        value: AttributeText<'a>,
    ) {
        attributes.insert(key.into(), value);
    }
}

pub trait NodeAttributes<'a> {
    // TODO: constrain
    /// Indicates the preferred area for a node or empty cluster when laid out by patchwork.
    /// default: 1.0, minimum: >0
    fn area(&mut self, area: f32) -> &mut Self {
        self.add_attribute("area", AttributeText::from(area))
    }

    /// Classnames to attach to the node’s SVG element.
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    fn class(&mut self, class: String) -> &mut Self {
        Attributes::class(self.get_attributes_mut(), class);
        self
    }

    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    fn color(&mut self, color: Color<'a>) -> &mut Self {
        Attributes::color(self.get_attributes_mut(), color);
        self
    }

    /// This attribute specifies a color scheme namespace: the context for interpreting color names.
    /// In particular, if a color value has form "xxx" or "//xxx", then the color xxx will be evaluated
    /// according to the current color scheme. If no color scheme is set, the standard X11 naming is used.
    /// For example, if colorscheme=bugn9, then color=7 is interpreted as color="/bugn9/7".
    fn color_scheme(&mut self, color_scheme: String) -> &mut Self {
        Attributes::color_scheme(self.get_attributes_mut(), color_scheme);
        self
    }

    /// Comments are inserted into output. Device-dependent
    fn comment(&mut self, comment: String) -> &mut Self {
        Attributes::comment(self.get_attributes_mut(), comment);
        self
    }

    /// Distortion factor for shape=polygon.
    /// Positive values cause top part to be larger than bottom; negative values do the opposite.
    fn distortion(&mut self, distortion: f32) -> &mut Self {
        self.add_attribute("distortion", AttributeText::from(distortion))
    }

    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    fn fill_color(&mut self, fill_color: Color<'a>) -> &mut Self {
        Attributes::fill_color(self.get_attributes_mut(), fill_color);
        self
    }

    /// Color used to fill the background, with a gradient, of a node or cluster assuming
    /// style=filled, or a filled arrowhead.
    fn fill_color_with_colorlist(&mut self, fill_colors: ColorList<'a>) -> &mut Self {
        Attributes::fill_color_with_colorlist(self.get_attributes_mut(), fill_colors);
        self
    }

    /// Color used to fill the background, with a gradient, of a node or cluster assuming
    /// style=filled, or a filled arrowhead.
    /// TODO: example
    fn fill_color_with_iter<I>(&mut self, fill_colors: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: IntoWeightedColor<'a>,
    {
        Attributes::fill_color_with_iter(self.get_attributes_mut(), fill_colors);
        self
    }

    /// If true, the node size is specified by the values of the width and height attributes only and
    /// is not expanded to contain the text label.
    /// There will be a warning if the label (with margin) cannot fit within these limits.
    /// If false, the size of a node is determined by smallest width and height needed to contain its label
    /// and image, if any, with a margin specified by the margin attribute.
    fn fixed_size(&mut self, fixed_size: bool) -> &mut Self {
        self.add_attribute("fixedsize", AttributeText::from(fixed_size))
    }

    /// Color used for text.
    fn font_color(&mut self, font_color: Color<'a>) -> &mut Self {
        Attributes::font_color(self.get_attributes_mut(), font_color);
        self
    }

    /// Font used for text.
    fn font_name(&mut self, font_name: String) -> &mut Self {
        Attributes::font_name(self.get_attributes_mut(), font_name);
        self
    }

    /// Font size, in points, used for text.
    /// default: 14.0, minimum: 1.0
    fn font_size(&mut self, font_size: f32) -> &mut Self {
        Attributes::font_size(self.get_attributes_mut(), font_size);
        self
    }

    /// If a gradient fill is being used, this determines the angle of the fill.
    fn gradient_angle(&mut self, gradient_angle: u32) -> &mut Self {
        Attributes::gradient_angle(self.get_attributes_mut(), gradient_angle);
        self
    }

    /// If the end points of an edge belong to the same group, i.e., have the same group attribute,
    /// parameters are set to avoid crossings and keep the edges straight.
    fn group(&mut self, group: String) -> &mut Self {
        self.add_attribute("group", AttributeText::attr(group))
    }

    // TODO: constrain
    /// Height of node, in inches.
    /// default: 0.5, minimum: 0.02
    fn height(&mut self, height: f32) -> &mut Self {
        self.add_attribute("height", AttributeText::from(height))
    }

    /// Gives the name of a file containing an image to be displayed inside a node.
    /// The image file must be in one of the recognized formats,
    /// typically JPEG, PNG, GIF, BMP, SVG, or Postscript, and be able to be converted
    /// into the desired output format.
    fn image(&mut self, image: String) -> &mut Self {
        self.add_attribute("image", AttributeText::quoted(image))
    }

    /// Controls how an image is positioned within its containing node.
    /// Only has an effect when the image is smaller than the containing node.
    fn image_pos(&mut self, image_pos: ImagePosition) -> &mut Self {
        self.add_attribute("imagepos", AttributeText::from(image_pos))
    }

    /// Controls how an image fills its containing node.
    fn image_scale_bool(&mut self, image_scale: bool) -> &mut Self {
        self.add_attribute("imagescale", AttributeText::from(image_scale))
    }

    /// Controls how an image fills its containing node.
    fn image_scale(&mut self, image_scale: ImageScale) -> &mut Self {
        self.add_attribute("imagescale", AttributeText::from(image_scale))
    }

    /// Text label attached to objects.
    fn label<S: Into<Cow<'a, str>>>(&mut self, text: S) -> &mut Self {
        self.add_attribute("label", AttributeText::quoted(text))
    }

    // Vertical placement of labels for nodes, root graphs and clusters.
    // For graphs and clusters, only labelloc=t and labelloc=b are allowed, corresponding to placement at the top and bottom, respectively.
    // By default, root graph labels go on the bottom and cluster labels go on the top.
    // Note that a subgraph inherits attributes from its parent. Thus, if the root graph sets labelloc=b, the subgraph inherits this value.
    // For nodes, this attribute is used only when the height of the node is larger than the height of its label.
    // If labelloc=t, labelloc=c, labelloc=b, the label is aligned with the top, centered, or aligned with the bottom of the node, respectively.
    // By default, the label is vertically centered.
    fn label_location(&mut self, label_location: LabelLocation) -> &mut Self {
        Attributes::label_location(self.get_attributes_mut(), label_location);
        self
    }

    /// Specifies layers in which the node, edge or cluster is present.
    fn layer(&mut self, layer: String) -> &mut Self {
        Attributes::layer(self.get_attributes_mut(), layer);
        self
    }

    /// Sets x and y margins of canvas, in inches.
    /// Both margins are set equal to the given value.
    /// See [`crate::NodeAttributes::margin_point`]
    fn margin(&mut self, margin: f32) -> &mut Self {
        self.margin_point(Point::new_2d(margin, margin))
    }

    /// Sets x and y margins of canvas, in inches.
    /// Specifies space left around the node’s label.
    /// Note that the margin is not part of the drawing but just empty space left around the drawing.
    /// The margin basically corresponds to a translation of drawing, as would be necessary to
    /// center a drawing on a page.
    /// Nothing is actually drawn in the margin.
    /// To actually extend the background of a drawing, see the pad attribute.
    /// Whilst it is possible to create a Point value with either a third co-ordinate
    /// or a forced position, these are ignored for printing.
    /// By default, the value is 0.11,0.055.
    fn margin_point(&mut self, margin: Point) -> &mut Self {
        Attributes::margin(self.get_attributes_mut(), margin);
        self
    }

    /// By default, the justification of multi-line labels is done within the largest context that makes sense.
    /// Thus, in the label of a polygonal node, a left-justified line will align with the left side of the node (shifted by the prescribed margin).
    /// In record nodes, left-justified line will line up with the left side of the enclosing column of fields.
    /// If nojustify=true, multi-line labels will be justified in the context of itself.
    /// For example, if nojustify is set, the first label line is long, and the second is shorter and left-justified,
    /// the second will align with the left-most character in the first line, regardless of how large the node might be.
    fn no_justify(&mut self, no_justify: bool) -> &mut Self {
        Attributes::no_justify(self.get_attributes_mut(), no_justify);
        self
    }

    /// If ordering="out", then the outedges of a node, that is, edges with the node as its tail
    /// node, must appear left-to-right in the same order in which they are defined in the input.
    ///
    /// If ordering="in", then the inedges of a node must appear left-to-right in the same order in
    /// which they are defined in the input.
    ///
    /// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph
    /// or subgraph.
    ///
    /// Note that the graph attribute takes precedence over the node attribute.
    fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        Attributes::ordering(self.get_attributes_mut(), ordering);
        self
    }

    // TODO: constrain to 0 - 360. Docs say min is 360 which should be max right?
    /// Angle, in degrees, to rotate polygon node shapes.
    /// For any number of polygon sides, 0 degrees rotation results in a flat base.
    /// Used only if rotate is not defined.
    /// Default: 0.0 and minimum: 360.0
    fn orientation(&mut self, orientation: f32) -> &mut Self {
        Attributes::orientation(self.get_attributes_mut(), orientation);
        self
    }

    /// Specifies the width of the pen, in points, used to draw lines and curves,
    /// including the boundaries of edges and clusters.
    /// default: 1.0, minimum: 0.0
    fn pen_width(&mut self, pen_width: f32) -> &mut Self {
        Attributes::pen_width(self.get_attributes_mut(), pen_width);
        self
    }

    /// Set number of peripheries used in polygonal shapes and cluster boundaries.
    fn peripheries(&mut self, peripheries: u32) -> &mut Self {
        self.add_attribute("penwidth", AttributeText::from(peripheries))
    }

    /// Position of node, or spline control points.
    /// the position indicates the center of the node. On output, the coordinates are in points.
    fn pos(&mut self, pos: Point) -> &mut Self {
        Attributes::pos(self.get_attributes_mut(), pos);
        self
    }

    // TODO: add post_spline

    /// Rectangles for fields of records, in points.
    fn rects(&mut self, rect: Rectangle) -> &mut Self {
        self.add_attribute("rects", AttributeText::from(rect))
    }

    /// If true, force polygon to be regular, i.e., the vertices of the polygon will
    /// lie on a circle whose center is the center of the node.
    fn regular(&mut self, regular: bool) -> &mut Self {
        self.add_attribute("regular", AttributeText::from(regular))
    }

    /// Gives the number of points used for a circle/ellipse node.
    fn sample_points(&mut self, sample_points: u32) -> &mut Self {
        self.add_attribute("samplepoints", AttributeText::from(sample_points))
    }

    /// Sets the shape of a node.
    fn shape(&mut self, shape: Shape) -> &mut Self {
        self.add_attribute("shape", AttributeText::from(shape))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at the end if showboxes=2.
    /// (Debugging, TB mode only!)
    /// default: 0, minimum: 0
    fn show_boxes(&mut self, show_boxes: u32) -> &mut Self {
        Attributes::show_boxes(self.get_attributes_mut(), show_boxes);
        self
    }

    /// Number of sides when shape=polygon.
    fn sides(&mut self, sides: u32) -> &mut Self {
        self.add_attribute("sides", AttributeText::from(sides))
    }

    // TODO: constrain
    /// Skew factor for shape=polygon.
    /// Positive values skew top of polygon to right; negative to left.
    /// default: 0.0, minimum: -100.0
    fn skew(&mut self, skew: f32) -> &mut Self {
        self.add_attribute("skew", AttributeText::from(skew))
    }

    /// If packmode indicates an array packing, sortv specifies an insertion order
    /// among the components, with smaller values inserted first.
    /// default: 0, minimum: 0
    fn sortv(&mut self, sortv: u32) -> &mut Self {
        Attributes::sortv(self.get_attributes_mut(), sortv);
        self
    }

    /// Set style information for components of the graph.
    fn style(&mut self, style: NodeStyle) -> &mut Self {
        Attributes::style(self.get_attributes_mut(), Styles::Node(style));
        self
    }

    /// If the object has a URL, this attribute determines which window of the browser is used for the URL.
    fn target(&mut self, target: String) -> &mut Self {
        Attributes::target(self.get_attributes_mut(), target);
        self
    }

    /// Tooltip annotation attached to the node or edge.
    /// If unset, Graphviz will use the object’s label if defined.
    /// Note that if the label is a record specification or an HTML-like label,
    /// the resulting tooltip may be unhelpful.
    /// In this case, if tooltips will be generated, the user should set a tooltip attribute explicitly.
    fn tooltip(&mut self, tooltip: String) -> &mut Self {
        Attributes::tooltip(self.get_attributes_mut(), tooltip);
        self
    }

    /// Hyperlinks incorporated into device-dependent output.
    fn url(&mut self, url: String) -> &mut Self {
        Attributes::url(self.get_attributes_mut(), url);
        self
    }

    /// Sets the coordinates of the vertices of the node’s polygon, in inches.
    /// A list of points, separated by spaces.
    fn vertices(&mut self, vertices: String) -> &mut Self {
        self.add_attribute("vertices", AttributeText::quoted(vertices))
    }

    /// Width of node, in inches.
    /// This is taken as the initial, minimum width of the node.
    /// If fixedsize is true, this will be the final width of the node.
    /// Otherwise, if the node label requires more width to fit, the node’s
    /// width will be increased to contain the label.
    fn width(&mut self, width: f32) -> &mut Self {
        self.add_attribute("width", AttributeText::from(width))
    }

    /// External label for a node or edge.
    /// The label will be placed outside of the node but near it.
    /// These labels are added after all nodes and edges have been placed.
    /// The labels will be placed so that they do not overlap any node or label.
    /// This means it may not be possible to place all of them.
    /// To force placing all of them, set forcelabels=true.
    fn xlabel(&mut self, xlabel: String) -> &mut Self {
        Attributes::xlabel(self.get_attributes_mut(), xlabel);
        self
    }

    /// Position of an exterior label, in points.
    /// The position indicates the center of the label.
    fn xlp(&mut self, xlp: Point) -> &mut Self {
        Attributes::xlp(self.get_attributes_mut(), xlp);
        self
    }

    /// Add an attribute to the node.
    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self;

    /// Add multiple attribures to the node.
    fn add_attributes(
        &'a mut self,
        attributes: HashMap<String, AttributeText<'a>>,
    ) -> &mut Self;

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>>;
}

pub trait EdgeAttributes<'a> {
    /// Style of arrowhead on the head node of an edge.
    /// This will only appear if the dir attribute is forward or both.
    fn arrow_head(&mut self, arrowhead: ArrowType) -> &mut Self {
        self.add_attribute("arrowhead", AttributeText::from(arrowhead))
    }

    // TODO: constrain
    /// Multiplicative scale factor for arrowheads.
    /// default: 1.0, minimum: 0.0
    fn arrow_size(&mut self, arrow_size: f32) -> &mut Self {
        self.add_attribute("arrowsize", AttributeText::from(arrow_size))
    }

    /// Style of arrowhead on the tail node of an edge.
    /// This will only appear if the dir attribute is back or both.
    fn arrowtail(&mut self, arrowtail: ArrowType) -> &mut Self {
        self.add_attribute("arrowtail", AttributeText::from(arrowtail))
    }

    /// Classnames to attach to the edge’s SVG element.
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    fn class(&mut self, class: String) -> &mut Self {
        Attributes::class(self.get_attributes_mut(), class);
        self
    }

    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    /// If any fraction is used, the colors are drawn in series, with each color being given
    /// roughly its specified fraction of the edge.
    fn color(&mut self, color: Color<'a>) -> &mut Self {
        Attributes::color(self.get_attributes_mut(), color);
        self
    }

    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    /// if colorList has no fractions, the edge is drawn using parallel splines or lines,
    /// one for each color in the list, in the order given.
    /// The head arrow, if any, is drawn using the first color in the list, and the tail
    /// arrow, if any, the second color. This supports the common case of drawing opposing edges,
    /// but using parallel splines instead of separately routed multiedges.
    /// If any fraction is used, the colors are drawn in series, with each color being given
    /// roughly its specified fraction of the edge.
    fn color_with_colorlist(&mut self, color: ColorList<'a>) -> &mut Self {
        Attributes::color_with_colorlist(self.get_attributes_mut(), color);
        self
    }

    /// This attribute specifies a color scheme namespace: the context for interpreting color names.
    /// In particular, if a color value has form "xxx" or "//xxx", then the color xxx will be evaluated
    /// according to the current color scheme. If no color scheme is set, the standard X11 naming is used.
    /// For example, if colorscheme=bugn9, then color=7 is interpreted as color="/bugn9/7".
    fn color_scheme(&mut self, color_scheme: String) -> &mut Self {
        Attributes::color_scheme(self.get_attributes_mut(), color_scheme);
        self
    }

    /// Comments are inserted into output. Device-dependent
    fn comment(&mut self, comment: String) -> &mut Self {
        self.add_attribute("comment", AttributeText::attr(comment));
        self
    }

    /// If false, the edge is not used in ranking the nodes.
    fn constraint(&mut self, constraint: bool) -> &mut Self {
        self.add_attribute("constraint", AttributeText::from(constraint))
    }

    /// If true, attach edge label to edge by a 2-segment polyline, underlining the label,
    /// then going to the closest point of spline.
    fn decorate(&mut self, decorate: bool) -> &mut Self {
        self.add_attribute("decorate", AttributeText::from(decorate))
    }

    /// Edge type for drawing arrowheads.
    /// Indicates which ends of the edge should be decorated with an arrowhead.
    /// The actual style of the arrowhead can be specified using the arrowhead and arrowtail attributes.
    fn dir(&mut self, dir: Direction) -> &mut Self {
        self.add_attribute("dir", AttributeText::from(dir))
    }

    /// If the edge has a URL or edgeURL attribute, edgetarget determines which window
    /// of the browser is used for the URL attached to the non-label part of the edge.
    /// Setting edgetarget=_graphviz will open a new window if it doesn’t already exist,
    /// or reuse it if it does.
    fn edge_target(&mut self, edge_target: String) -> &mut Self {
        self.add_attribute("edgetarget", AttributeText::escaped(edge_target))
    }

    /// Tooltip annotation attached to the non-label part of an edge.
    /// Used only if the edge has a URL or edgeURL attribute.
    fn edge_tooltip(&mut self, edge_tooltip: String) -> &mut Self {
        self.add_attribute("edgetooltip", AttributeText::escaped(edge_tooltip))
    }

    /// The link for the non-label parts of an edge.
    /// edgeURL overrides any URL defined for the edge.
    /// Also, edgeURL is used near the head or tail node unless overridden by headURL or tailURL, respectively.
    fn edge_url(&mut self, edge_url: String) -> &mut Self {
        self.add_attribute("edgeurl", AttributeText::escaped(edge_url))
    }

    // TODO: color list
    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    fn fill_color(&mut self, fill_color: Color<'a>) -> &mut Self {
        Attributes::fill_color(self.get_attributes_mut(), fill_color);
        self
    }

    // TODO: color list
    /// Color used for text.
    fn font_color(&mut self, font_color: Color<'a>) -> &mut Self {
        Attributes::font_color(self.get_attributes_mut(), font_color);
        self
    }

    /// Font used for text.
    fn font_name(&mut self, font_name: String) -> &mut Self {
        Attributes::font_name(self.get_attributes_mut(), font_name);
        self
    }

    /// Font size, in points, used for text.
    /// default: 14.0, minimum: 1.0
    fn font_size(&mut self, font_size: f32) -> &mut Self {
        Attributes::font_size(self.get_attributes_mut(), font_size);
        self
    }

    /// Position of an edge’s head label, in points. The position indicates the center of the label.
    fn head_lp(&mut self, head_lp: Point) -> &mut Self {
        self.add_attribute("head_lp", AttributeText::from(head_lp))
    }

    /// If true, the head of an edge is clipped to the boundary of the head node;
    /// otherwise, the end of the edge goes to the center of the node, or the center
    /// of a port, if applicable.
    fn head_clip(&mut self, head_clip: bool) -> &mut Self {
        self.add_attribute("headclip", AttributeText::from(head_clip))
    }

    /// Text label to be placed near head of edge.
    fn head_label(&mut self, head_label: String) -> &mut Self {
        self.add_attribute("headlabel", AttributeText::quoted(head_label))
    }

    /// Indicates where on the head node to attach the head of the edge.
    /// In the default case, the edge is aimed towards the center of the node,
    /// and then clipped at the node boundary.
    fn head_port(&mut self, head_port: PortPosition) -> &mut Self {
        self.add_attribute("headport", AttributeText::from(head_port))
    }

    /// If the edge has a headURL, headtarget determines which window of the browser is used for the URL.
    /// Setting headURL=_graphviz will open a new window if the window doesn’t already exist,
    /// or reuse the window if it does.
    /// If undefined, the value of the target is used.
    fn head_target(&mut self, head_target: String) -> &mut Self {
        self.add_attribute("headtarget", AttributeText::escaped(head_target))
    }

    /// Tooltip annotation attached to the head of an edge.
    /// Used only if the edge has a headURL attribute.
    fn head_tooltip(&mut self, head_tooltip: String) -> &mut Self {
        self.add_attribute("headtooltip", AttributeText::escaped(head_tooltip))
    }

    /// If defined, headURL is output as part of the head label of the edge.
    /// Also, this value is used near the head node, overriding any URL value.
    fn head_url(&mut self, head_url: String) -> &mut Self {
        self.add_attribute("headURL", AttributeText::escaped(head_url))
    }

    /// An escString or an HTML label.
    fn label(&mut self, label: String) -> &mut Self {
        Attributes::label(self.get_attributes_mut(), label);
        self
    }

    // TODO: constrain
    /// Determines, along with labeldistance, where the headlabel / taillabel are
    /// placed with respect to the head / tail in polar coordinates.
    /// The origin in the coordinate system is the point where the edge touches the node.
    /// The ray of 0 degrees goes from the origin back along the edge, parallel to the edge at the origin.
    /// The angle, in degrees, specifies the rotation from the 0 degree ray,
    /// with positive angles moving counterclockwise and negative angles moving clockwise.
    /// default: -25.0, minimum: -180.0
    fn label_angle(&mut self, label_angle: f32) -> &mut Self {
        self.add_attribute("labelangle", AttributeText::from(label_angle))
    }

    /// Multiplicative scaling factor adjusting the distance that the headlabel / taillabel is from
    /// the head / tail node.
    /// default: 1.0, minimum: 0.0
    fn label_distance(&mut self, label_distance: f32) -> &mut Self {
        self.add_attribute("labeldistance", AttributeText::from(label_distance))
    }

    /// If true, allows edge labels to be less constrained in position.
    /// In particular, it may appear on top of other edges.
    fn label_float(&mut self, label_float: bool) -> &mut Self {
        self.add_attribute("labelfloat", AttributeText::from(label_float))
    }

    /// Color used for headlabel and taillabel.
    fn label_font_color(&mut self, label_font_color: Color<'a>) -> &mut Self {
        self.add_attribute("labelfontcolor", AttributeText::from(label_font_color))
    }

    /// Font used for headlabel and taillabel.
    /// If not set, defaults to edge’s fontname.
    fn label_font_name(&mut self, label_font_name: String) -> &mut Self {
        self.add_attribute("labelfontname", AttributeText::attr(label_font_name))
    }

    // TODO: constrains
    /// Font size, in points, used for headlabel and taillabel.
    /// If not set, defaults to edge’s fontsize.
    /// default: 14.0, minimum: 1.0
    fn label_font_size(&mut self, label_font_size: f32) -> &mut Self {
        self.add_attribute("labelfontsize", AttributeText::from(label_font_size))
    }

    /// If the edge has a URL or labelURL attribute, this attribute determines
    ///  which window of the browser is used for the URL attached to the label.
    fn label_target(&mut self, label_target: String) -> &mut Self {
        self.add_attribute("labeltarget", AttributeText::escaped(label_target))
    }

    /// Tooltip annotation attached to label of an edge.
    /// Used only if the edge has a URL or labelURL attribute.
    fn label_tooltip(&mut self, label_tooltip: String) -> &mut Self {
        self.add_attribute("labeltooltip", AttributeText::escaped(label_tooltip))
    }

    /// If defined, labelURL is the link used for the label of an edge.
    /// labelURL overrides any URL defined for the edge.
    fn label_url(&mut self, label_url: String) -> &mut Self {
        self.add_attribute("labelurl", AttributeText::escaped(label_url))
    }

    fn layer(&mut self, layer: String) -> &mut Self {
        Attributes::layer(self.get_attributes_mut(), layer);
        self
    }

    fn lhead(&mut self, lhead: String) -> &mut Self {
        self.add_attribute("lhead", AttributeText::quoted(lhead))
    }

    /// Label position
    /// The position indicates the center of the label.
    fn label_position(&mut self, lp: Point) -> &mut Self {
        Attributes::label_position(self.get_attributes_mut(), lp);
        self
    }

    /// Logical tail of an edge.
    /// When compound=true, if ltail is defined and is the name of a cluster
    /// containing the real tail, the edge is clipped to the boundary of the cluster.
    fn ltail(&mut self, ltail: String) -> &mut Self {
        self.add_attribute("ltail", AttributeText::quoted(ltail))
    }

    /// Minimum edge length (rank difference between head and tail).
    fn min_len(&mut self, min_len: u32) -> &mut Self {
        self.add_attribute("minlen", AttributeText::from(min_len))
    }

    fn no_justify(&mut self, no_justify: bool) -> &mut Self {
        self.add_attribute("nojustify", AttributeText::from(no_justify))
    }

    fn pen_width(&mut self, pen_width: f32) -> &mut Self {
        Attributes::pen_width(self.get_attributes_mut(), pen_width);
        self
    }

    /// Position of node, or spline control points.
    /// the position indicates the center of the node. On output, the coordinates are in points.
    fn pos(&mut self, pos: Point) -> &mut Self {
        Attributes::pos(self.get_attributes_mut(), pos);
        self
    }

    /// Edges with the same head and the same samehead value are aimed at the same point on the head.
    fn same_head(&mut self, same_head: String) -> &mut Self {
        self.add_attribute("samehead", AttributeText::quoted(same_head))
    }

    /// Edges with the same tail and the same sametail value are aimed at the same point on the tail.
    fn same_tail(&mut self, same_tail: String) -> &mut Self {
        self.add_attribute("sametail", AttributeText::quoted(same_tail))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at the
    /// end if showboxes=2.
    /// (Debugging, TB mode only!)
    /// default: 0, minimum: 0
    fn show_boxes(&mut self, show_boxes: u32) -> &mut Self {
        Attributes::show_boxes(self.get_attributes_mut(), show_boxes);
        self
    }

    /// Set style information for components of the graph.
    fn style(&mut self, style: EdgeStyle) -> &mut Self {
        Attributes::style(self.get_attributes_mut(), Styles::Edge(style));
        self
    }

    /// Position of an edge’s tail label, in points.
    /// The position indicates the center of the label.
    fn tail_lp(&mut self, tail_lp: Point) -> &mut Self {
        self.add_attribute("tail_lp", AttributeText::from(tail_lp))
    }

    /// If true, the tail of an edge is clipped to the boundary of the tail node; otherwise,
    /// the end of the edge goes to the center of the node, or the center of a port, if applicable.
    fn tail_clip(&mut self, tail_clip: bool) -> &mut Self {
        self.add_attribute("tailclip", AttributeText::from(tail_clip))
    }

    /// Text label to be placed near tail of edge.
    fn tail_label(&mut self, tail_label: String) -> &mut Self {
        self.add_attribute("taillabel", AttributeText::quoted(tail_label))
    }

    /// Indicates where on the tail node to attach the tail of the edge.
    fn tail_port(&mut self, tail_port: PortPosition) -> &mut Self {
        self.add_attribute("tailport", AttributeText::from(tail_port))
    }

    /// If the edge has a tailURL, tailtarget determines which window of the browser is used for the URL.
    fn tail_target(&mut self, tail_target: String) -> &mut Self {
        self.add_attribute("tailtarget", AttributeText::escaped(tail_target))
    }

    /// Tooltip annotation attached to the tail of an edge.
    fn tail_tooltip(&mut self, tail_tooltip: String) -> &mut Self {
        self.add_attribute("tailtooltip", AttributeText::escaped(tail_tooltip))
    }

    /// If defined, tailURL is output as part of the tail label of the edge.
    /// Also, this value is used near the tail node, overriding any URL value.
    fn tail_url(&mut self, tail_url: String) -> &mut Self {
        self.add_attribute("tailURL", AttributeText::escaped(tail_url))
    }

    /// If the object has a URL, this attribute determines which window of the browser is used for the URL.
    fn target(&mut self, target: String) -> &mut Self {
        self.add_attribute("target", AttributeText::escaped(target))
    }

    /// Tooltip annotation attached to the node or edge.
    /// If unset, Graphviz will use the object’s label if defined.
    /// Note that if the label is a record specification or an HTML-like label,
    /// the resulting tooltip may be unhelpful.
    /// In this case, if tooltips will be generated, the user should set a tooltip attribute explicitly.
    fn tooltip(&mut self, tooltip: String) -> &mut Self {
        Attributes::tooltip(self.get_attributes_mut(), tooltip);
        self
    }

    /// Hyperlinks incorporated into device-dependent output.
    fn url(&mut self, url: String) -> &mut Self {
        Attributes::url(self.get_attributes_mut(), url);
        self
    }

    // TODO: contrain
    /// Weight of edge.
    /// The heavier the weight, the shorter, straighter and more vertical the edge is.
    /// default: 1, minimum: 0
    fn weight(&mut self, weight: u32) -> &mut Self {
        self.add_attribute("weight", AttributeText::attr(weight.to_string()))
    }

    /// External label for a node or edge.
    /// The label will be placed outside of the node but near it.
    /// These labels are added after all nodes and edges have been placed.
    /// The labels will be placed so that they do not overlap any node or label.
    /// This means it may not be possible to place all of them.
    /// To force placing all of them, set forcelabels=true.
    fn xlabel(&mut self, xlabel: String) -> &mut Self {
        Attributes::xlabel(self.get_attributes_mut(), xlabel);
        self
    }

    /// Position of an exterior label, in points.
    /// The position indicates the center of the label.
    fn xlp(&mut self, xlp: Point) -> &mut Self {
        Attributes::xlp(self.get_attributes_mut(), xlp);
        self
    }

    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self;

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>>;

    // fn add_attribute<S: Into<String>>(
    //     &self,
    //     key: S,
    //     value: AttributeText<'a>
    // ) {
    //     self.get_attributes().insert(key.into(), value);
    // }

    // fn get_attributes(&self) -> IndexMap<String, AttributeText<'a>>;

    // fn get_attributes_mut(&self) -> &mut IndexMap<String, AttributeText<'a>>;

    // fn to_dot_string(&self) -> String;
}

pub(crate) fn fmt_attributes(attributes: &IndexMap<String, AttributeText>) -> String {
    let mut dot_string = String::from("");
    if !attributes.is_empty() {
        dot_string.push_str(" [");
        let mut iter = attributes.iter();
        let first = iter.next().unwrap();
        dot_string.push_str(format!("{}={}", first.0, first.1.dot_string()).as_str());
        for (key, value) in iter {
            dot_string.push_str(", ");
            dot_string.push_str(format!("{}={}", key, value.dot_string()).as_str());
        }
        dot_string.push_str("]");
    }

    dot_string
}

#[cfg(test)]
mod test {
    use crate::attributes::{
        AttributeStatement, Color, GraphAttributeStatementBuilder, GraphAttributes,
    };

    #[test]
    fn graph_attribute_colorlist_vec_dot_string() {
        let graph_attributes = GraphAttributeStatementBuilder::new()
            .fill_color_with_iter(&[
                (Color::Named("yellow"), Some(0.3)),
                (Color::Named("blue"), None),
            ])
            .build();

        assert_eq!(
            "graph [fillcolor=\"yellow;0.3:blue\"];",
            graph_attributes.dot_string()
        );
    }
}
