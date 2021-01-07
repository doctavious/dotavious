use crate::attributes::{
    ArrowType, AttributeStatement, AttributeText, AttributeType, Attributes, Color,
    ColorList, Direction, EdgeStyle, GraphAttributeStatement, ImagePosition, ImageScale,
    IntoWeightedColor, LabelLocation, NodeStyle, Ordering, Point, PortPosition,
    Rectangle, Shape, Styles,
};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;

static INDENT: &str = "    ";

pub trait DotString<'a> {
    fn dot_string(&self) -> Cow<'a, str>;
}

// TODO: probably dont need this struct and can move impl methods into lib module
pub struct Dot<'a> {
    pub graph: Graph<'a>,
}

impl<'a> Dot<'a> {
    /// Renders directed graph `g` into the writer `w` in DOT syntax.
    /// (Simple wrapper around `render_opts` that passes a default set of options.)
    //pub fn render<W>(self, g: Graph, w: &mut W) -> io::Result<()>
    pub fn render<W>(self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        // TODO: use default_options?
        self.render_opts(w, &[])
    }

    // io::Result<()> vs Result<(), Box<dyn Error>>
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-can-be-used-in-functions-that-return-result
    /// Renders directed graph `g` into the writer `w` in DOT syntax.
    /// (Main entry point for the library.)
    // pub fn render_opts<W>(self, graph: Graph, w: &mut W, options: &[RenderOption]) -> io::Result<()>
    pub fn render_opts<W>(self, w: &mut W, options: &[RenderOption]) -> io::Result<()>
    where
        W: Write,
    {
        if let Some(comment) = &self.graph.comment {
            // TODO: split comment into lines of 80 or so characters
            writeln!(w, "// {}", comment)?;
        }

        let edge_op = &self.graph.edge_op();
        let strict = if self.graph.strict { "strict " } else { "" };
        write!(w, "{}{}", strict, &self.graph.graph_type())?;

        if let Some(id) = &self.graph.id {
            write!(w, " {}", id)?;
        }

        writeln!(w, " {{")?;

        if let Some(graph_attributes) = self.graph.graph_attributes {
            write!(w, "{}{}\n", INDENT, graph_attributes.dot_string())?;
        }

        if let Some(node_attributes) = self.graph.node_attributes {
            write!(w, "{}{}\n", INDENT, node_attributes.dot_string())?;
        }

        if let Some(edge_attributes) = self.graph.edge_attributes {
            write!(w, "{}{}\n", INDENT, edge_attributes.dot_string())?;
        }

        for n in self.graph.nodes {
            // TODO: handle render options
            // Are render options something we need?
            // we could clone the node or and remove the attributes based on render options
            // or maybe we keep a set of attributes to ignore based on the options
            writeln!(w, "{}{}", INDENT, n.dot_string())?;
        }

        for e in self.graph.edges {
            let mut edge_source = e.source;
            if let Some(source_port_position) = e.source_port_position {
                edge_source
                    .push_str(format!(":{}", source_port_position.dot_string()).as_str())
            }

            let mut edge_target = e.target;
            if let Some(target_port_position) = e.target_port_position {
                edge_target
                    .push_str(format!(":{}", target_port_position.dot_string()).as_str())
            }

            write!(w, "{}{} {} {}", INDENT, edge_source, edge_op, edge_target)?;
            // TODO: render ops
            if !e.attributes.is_empty() {
                write!(w, " [")?;

                let mut iter = e.attributes.iter();
                let first = iter.next().unwrap();
                write!(w, "{}={}", first.0, first.1.dot_string())?;
                for (key, value) in iter {
                    write!(w, ", ")?;
                    write!(w, "{}={}", key, value.dot_string())?;
                }
                write!(w, "]")?;
            }
            writeln!(w, ";")?;
        }

        writeln!(w, "}}")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RenderOption {
    NoEdgeLabels,
    NoNodeLabels,
    NoEdgeStyles,
    NoNodeStyles,

    // TODO: replace with Fontname(String),
    Monospace,
}

/// Returns vec holding all the default render options.
pub fn default_options() -> Vec<RenderOption> {
    vec![]
}

pub struct Graph<'a> {
    pub id: Option<String>,

    pub is_directed: bool,

    pub strict: bool,

    /// Comment added to the first line of the source.
    pub comment: Option<String>,

    pub graph_attributes: Option<GraphAttributeStatement<'a>>,

    pub node_attributes: Option<NodeAttributeStatement<'a>>,

    pub edge_attributes: Option<EdgeAttributeStatement<'a>>,

    pub nodes: Vec<Node<'a>>,

    pub edges: Vec<Edge<'a>>,
}

impl<'a> Graph<'a> {
    pub fn new(
        id: Option<String>,
        is_directed: bool,
        strict: bool,
        comment: Option<String>,
        graph_attributes: Option<GraphAttributeStatement<'a>>,
        node_attributes: Option<NodeAttributeStatement<'a>>,
        edge_attributes: Option<EdgeAttributeStatement<'a>>,
        nodes: Vec<Node<'a>>,
        edges: Vec<Edge<'a>>,
    ) -> Self {
        Self {
            id,
            is_directed,
            strict,
            comment,
            graph_attributes,
            node_attributes,
            edge_attributes,
            nodes,
            edges,
        }
    }

    pub fn graph_type(&self) -> &'static str {
        if self.is_directed {
            "digraph"
        } else {
            "graph"
        }
    }

    pub fn edge_op(&self) -> &'static str {
        if self.is_directed {
            "->"
        } else {
            "--"
        }
    }
}

pub struct GraphBuilder<'a> {
    id: Option<String>,

    is_directed: bool,

    strict: bool,

    graph_attributes: Option<GraphAttributeStatement<'a>>,

    node_attributes: Option<NodeAttributeStatement<'a>>,

    edge_attributes: Option<EdgeAttributeStatement<'a>>,

    nodes: Vec<Node<'a>>,

    edges: Vec<Edge<'a>>,

    comment: Option<String>,
}

// TODO: id should be an escString
impl<'a> GraphBuilder<'a> {
    pub fn new_directed(id: Option<String>) -> Self {
        Self {
            id,
            is_directed: true,
            strict: false,
            graph_attributes: None,
            node_attributes: None,
            edge_attributes: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            comment: None,
        }
    }

    pub fn new_undirected(id: Option<String>) -> Self {
        Self {
            id,
            is_directed: false,
            strict: false,
            graph_attributes: None,
            node_attributes: None,
            edge_attributes: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            comment: None,
        }
    }

    pub fn comment<S: Into<String>>(&mut self, comment: S) -> &mut Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn add_graph_attributes(
        &mut self,
        graph_attributes: GraphAttributeStatement<'a>,
    ) -> &mut Self {
        self.graph_attributes = Some(graph_attributes);
        self
    }

    pub fn add_node_attributes(
        &mut self,
        node_attributes: NodeAttributeStatement<'a>,
    ) -> &mut Self {
        self.node_attributes = Some(node_attributes);
        self
    }

    pub fn add_edge_attributes(
        &mut self,
        edge_attributes: EdgeAttributeStatement<'a>,
    ) -> &mut Self {
        self.edge_attributes = Some(edge_attributes);
        self
    }

    // TODO: update to insert into appropriate statement or remove?
    // pub fn add_attribute(
    //     &mut self,
    //     attribute_type: AttributeType,
    //     key: String, value: AttributeText<'a>
    // ) -> &mut Self {
    //     self.get_attributes(attribute_type).insert(key, value);
    //     self
    // }
    //
    // pub fn add_attributes(
    //     &mut self,
    //     attribute_type: AttributeType,
    //     attributes: HashMap<String, AttributeText<'a>>
    // ) -> &mut Self {
    //     self.get_attributes(attribute_type).extend(attributes);
    //     self
    // }

    pub fn add_attribute(
        &mut self,
        attribute_type: AttributeType,
        key: String,
        value: AttributeText<'a>,
    ) -> &mut Self {
        match attribute_type {
            AttributeType::Graph => {
                if self.graph_attributes.is_none() {
                    self.graph_attributes = Some(GraphAttributeStatement::new());
                }
                self.graph_attributes
                    .as_mut()
                    .unwrap()
                    .add_attribute(key, value);
            }
            AttributeType::Edge => {
                if self.edge_attributes.is_none() {
                    self.edge_attributes = Some(EdgeAttributeStatement::new());
                }
                self.edge_attributes
                    .as_mut()
                    .unwrap()
                    .add_attribute(key, value);
            }
            AttributeType::Node => {
                if self.node_attributes.is_none() {
                    self.node_attributes = Some(NodeAttributeStatement::new());
                }
                self.node_attributes
                    .as_mut()
                    .unwrap()
                    .add_attribute(key, value);
            }
        }
        self
    }

    pub fn add_node(&mut self, node: Node<'a>) -> &mut Self {
        self.nodes.push(node);
        self
    }

    pub fn add_edge(&mut self, edge: Edge<'a>) -> &mut Self {
        self.edges.push(edge);
        self
    }

    pub fn strict(&mut self) -> &mut Self {
        self.strict = true;
        self
    }

    pub fn build(&self) -> Graph<'a> {
        Graph {
            id: self.id.to_owned(),
            is_directed: self.is_directed,
            strict: self.strict,
            comment: self.comment.clone(), // TODO: is clone the only option here?
            graph_attributes: self.graph_attributes.clone(),
            node_attributes: self.node_attributes.clone(),
            edge_attributes: self.edge_attributes.clone(),
            nodes: self.nodes.clone(), // TODO: is clone the only option here?
            edges: self.edges.clone(), // TODO: is clone the only option here?
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node<'a> {
    pub id: String,
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> Node<'a> {
    pub fn new(id: String) -> Node<'a> {
        // TODO: constrain id
        Node {
            id,
            attributes: IndexMap::new(),
        }
    }
}

impl<'a> DotString<'a> for Node<'a> {
    fn dot_string(&self) -> Cow<'a, str> {
        let mut dot_string = format!("{}", &self.id);
        if !self.attributes.is_empty() {
            dot_string.push_str(" [");
            let mut iter = self.attributes.iter();
            let first = iter.next().unwrap();
            dot_string
                .push_str(format!("{}={}", first.0, first.1.dot_string()).as_str());
            for (key, value) in iter {
                dot_string.push_str(", ");
                dot_string.push_str(format!("{}={}", key, value.dot_string()).as_str());
            }

            dot_string.push_str("]");
        }
        dot_string.push_str(";");
        dot_string.into()
    }
}

pub struct NodeBuilder<'a> {
    id: String,
    attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> NodeAttributes<'a> for NodeBuilder<'a> {
    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the edge.
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

impl<'a> NodeBuilder<'a> {
    pub fn new(id: String) -> Self {
        Self {
            id,
            attributes: IndexMap::new(),
        }
    }

    pub fn build(&self) -> Node<'a> {
        Node {
            // TODO: are these to_owned and clones necessary?
            id: self.id.to_owned(),
            attributes: self.attributes.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edge<'a> {
    pub source: String,
    pub source_port_position: Option<PortPosition>,
    pub target: String,
    pub target_port_position: Option<PortPosition>,
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> Edge<'a> {
    pub fn new(source: String, target: String) -> Self {
        Self {
            source,
            source_port_position: None,
            target,
            target_port_position: None,
            attributes: IndexMap::new(),
        }
    }

    pub fn new_with_position(
        source: String,
        source_port_position: PortPosition,
        target: String,
        target_port_position: PortPosition,
    ) -> Self {
        Self {
            source,
            source_port_position: Some(source_port_position),
            target,
            target_port_position: Some(target_port_position),
            attributes: IndexMap::new(),
        }
    }
}

pub struct EdgeBuilder<'a> {
    pub source: String,
    pub source_port_position: Option<PortPosition>,
    pub target: String,
    pub target_port_position: Option<PortPosition>,
    attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> EdgeAttributes<'a> for EdgeBuilder<'a> {
    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>> {
        &mut self.attributes
    }

    // /// Add multiple attributes to the edge.
    // fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
    //     self.attributes.extend(attributes);
    //     self
    // }
}

impl<'a> EdgeBuilder<'a> {
    pub fn new(source: String, target: String) -> Self {
        Self {
            source,
            target,
            source_port_position: None,
            target_port_position: None,
            attributes: IndexMap::new(),
        }
    }

    pub fn new_with_port_position(
        source: String,
        source_port_position: PortPosition,
        target: String,
        target_port_position: PortPosition,
    ) -> Self {
        Self {
            source,
            target,
            source_port_position: Some(source_port_position),
            target_port_position: Some(target_port_position),
            attributes: IndexMap::new(),
        }
    }

    pub fn source_port_position(&mut self, port_position: PortPosition) -> &mut Self {
        self.source_port_position = Some(port_position);
        self
    }

    pub fn target_port_position(&mut self, port_position: PortPosition) -> &mut Self {
        self.target_port_position = Some(port_position);
        self
    }
    /// Add an attribute to the edge.
    pub fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attributes to the edge.
    pub fn add_attributes(
        &'a mut self,
        attributes: HashMap<String, AttributeText<'a>>,
    ) -> &mut Self {
        self.attributes.extend(attributes);
        self
    }

    pub fn build(&self) -> Edge<'a> {
        Edge {
            // TODO: are these to_owned and clones necessary?
            source: self.source.to_owned(),
            source_port_position: self.source_port_position.to_owned(),
            target: self.target.to_owned(),
            target_port_position: self.target_port_position.to_owned(),
            attributes: self.attributes.clone(),
        }
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
    /// When used on nodes: Angle, in degrees, to rotate polygon node shapes.
    /// For any number of polygon sides, 0 degrees rotation results in a flat base.
    /// When used on graphs: If "[lL]*", sets graph orientation to landscape.
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

impl<'a> NodeAttributes<'a> for NodeAttributeStatementBuilder<'a> {
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
pub struct NodeAttributeStatementBuilder<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> NodeAttributeStatementBuilder<'a> {
    pub fn new() -> Self {
        Self {
            attributes: IndexMap::new(),
        }
    }

    pub fn build(&self) -> NodeAttributeStatement<'a> {
        NodeAttributeStatement {
            attributes: self.attributes.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeAttributeStatement<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> NodeAttributeStatement<'a> {
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

impl<'a> AttributeStatement<'a> for NodeAttributeStatement<'a> {
    fn get_attribute_statement_type(&self) -> &'static str {
        "node"
    }

    fn get_attributes(&self) -> &IndexMap<String, AttributeText<'a>> {
        &self.attributes
    }
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
    fn color(&mut self, color: Color<'a>) -> &mut Self {
        Attributes::color(self.get_attributes_mut(), color);
        self
    }

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
    fn constriant(&mut self, constriant: bool) -> &mut Self {
        self.add_attribute("constriant", AttributeText::from(constriant))
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

impl<'a> EdgeAttributes<'a> for EdgeAttributeStatementBuilder<'a> {
    fn add_attribute<S: Into<String>>(
        &mut self,
        key: S,
        value: AttributeText<'a>,
    ) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>> {
        &mut self.attributes
    }
}

impl<'a> AttributeStatement<'a> for EdgeAttributeStatement<'a> {
    fn get_attribute_statement_type(&self) -> &'static str {
        "edge"
    }

    fn get_attributes(&self) -> &IndexMap<String, AttributeText<'a>> {
        &self.attributes
    }
}

// I'm not a huge fan of needing this builder but having a hard time getting around &mut without it
pub struct EdgeAttributeStatementBuilder<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> EdgeAttributeStatementBuilder<'a> {
    pub fn new() -> Self {
        Self {
            attributes: IndexMap::new(),
        }
    }

    pub fn build(&self) -> EdgeAttributeStatement<'a> {
        EdgeAttributeStatement {
            attributes: self.attributes.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdgeAttributeStatement<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> EdgeAttributeStatement<'a> {
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
