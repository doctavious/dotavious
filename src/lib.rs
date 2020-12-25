//! Simple graphviz dot file format output.

use AttributeText::*;
use indexmap::IndexMap;
use std;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;

static INDENT: &str = "    ";

// TODO: support adding edge based on index of nodes?
// TODO: support fluent graph builder methods
// TODO: handle render options
// TODO: explicit attribute methods with type safety and enforce constraints
// i'm thinking we have NodeTraits/GraphTraints/EdgeTraits (what about none? is that a graph trait?)
// which will have default methods that use an associated type field called "state" or "attribtues" etc
// TODO: implement Clone for Graph
// TODO: remove duplicate fns
// TODO: Node port and compass
// TODO: <S: Into<String>> / <S: Into<Cow<'a, str>>
// TODO: see if we can get any insights from Haskell implementation
// https://hackage.haskell.org/package/graphviz-2999.20.1.0/docs/Data-GraphViz-Attributes-Complete.html#t:Point
// - I like this: A summary of known current constraints/limitations/differences:
// Has GlobalAttributes which hs GraphAttrs, NodeAttrs, EdgeAttrs

/// Most of this comes from core rust. Where to provide attribution?
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
    QuottedStr(Cow<'a, str>),
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

    pub fn quotted<S: Into<Cow<'a, str>>>(s: S) -> AttributeText<'a> {
        QuottedStr(s.into())
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
    pub fn to_dot_string(&self) -> String {
        match *self {
            AttrStr(ref s) => format!("{}", s),
            EscStr(ref s) => format!("\"{}\"", AttributeText::escape_str(&s)),
            HtmlStr(ref s) => format!("<{}>", s),
            QuottedStr(ref s) => format!("\"{}\"", s.escape_default()),
        }
    }

    /// Decomposes content into string suitable for making EscStr that
    /// yields same content as self. The result obeys the law
    /// render(`lt`) == render(`EscStr(lt.pre_escaped_content())`) for
    /// all `lt: LabelText`.
    fn pre_escaped_content(self) -> Cow<'a, str> {
        match self {
            AttrStr(s) => s,
            EscStr(s) => s,
            HtmlStr(s) => s,
            QuottedStr(s) => {
                if s.contains('\\') {
                    (&*s).escape_default().to_string().into()
                } else {
                    s
                }
            }
            
        }
    }

    /// Puts `prefix` on a line above this label, with a blank line separator.
    pub fn prefix_line(self, prefix: AttributeText<'_>) -> AttributeText<'static> {
        prefix.suffix_line(self)
    }

    /// Puts `suffix` on a line below this label, with a blank line separator.
    pub fn suffix_line(self, suffix: AttributeText<'_>) -> AttributeText<'static> {
        let mut prefix = self.pre_escaped_content().into_owned();
        let suffix = suffix.pre_escaped_content();
        prefix.push_str(r"\n\n");
        prefix.push_str(&suffix);
        EscStr(prefix.into())
    }
}

// TODO: need a way to print out values
// TODO: not sure we need this enum but should support setting nodeport either via
// headport / tailport attributes e.g. a -> b [tailport=se]
// or via edge declaration using the syntax node name:port_name e.g. a -> b:se
// aka compass
pub enum Compass {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    None
}

impl Compass {
    pub fn as_slice(self) -> &'static str {
        match self {
            Compass::N => "n",
            Compass::NE => "ne",
            Compass::E => "e",
            Compass::SE => "se",
            Compass::S => "s",
            Compass::SW => "sw",
            Compass::W => "w",
            Compass::NW => "nw",
            Compass::None => "",
        }
    }
}

// TODO: probably dont need this struct and can move impl methods into lib module
// pub struct Dot<'a, Ty> {
pub struct Dot<'a> {
    // graph: Graph<'a, Ty>,
    graph: Graph<'a>
    //config: Config,
}

// impl<'a, Ty> Dot<'a, Ty> 
// where Ty: GraphType
impl<'a> Dot<'a>
{

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

        let strict = if self.graph.strict { "strict " } else { "" }; 
        let id = self.graph.id.as_deref().unwrap_or_default();
        let edge_op = self.graph.edge_op();

        writeln!(w, "{}{} {} {{", strict, &self.graph.graph_type(), id)?;

        if let Some(graph_attributes) = self.graph.graph_attributes {
            write!(w, "{}", graph_attributes.to_dot_string())?;
        }

        if let Some(node_attributes) = self.graph.node_attributes {
            write!(w, "{}", node_attributes.to_dot_string())?;
        }

        if let Some(edge_attributes) = self.graph.edge_attributes {
            write!(w, "{}", edge_attributes.to_dot_string())?;
        }

        // TODO: clean this up
        for (key, value) in self.graph.attributes {
            write!(w, "{}", INDENT)?;
            match key {
                AttributeType::Edge => {
                    write!(w, "edge")?;
                    if !value.is_empty() {
                        write!(w, " [")?;
        
                        let mut iter = value.iter();
                        let first = iter.next().unwrap();
                        write!(w, "{}={}", first.0, first.1.to_dot_string())?;
                        for (key, value) in iter {
                            write!(w, ", ")?;
                            write!(w, "{}={}", key, value.to_dot_string())?;
                        }
                        write!(w, "]")?;
                    }
                    writeln!(w, ";")?;
                },
                AttributeType::Graph => {
                    write!(w, "graph")?;
                    if !value.is_empty() {
                        write!(w, " [")?;
        
                        let mut iter = value.iter();
                        let first = iter.next().unwrap();
                        write!(w, "{}={}", first.0, first.1.to_dot_string())?;
                        for (key, value) in iter {
                            write!(w, ", ")?;
                            write!(w, "{}={}", key, value.to_dot_string())?;
                        }
                        write!(w, "]")?;
                    }
                    writeln!(w, ";")?;
                },
                AttributeType::Node => {
                    write!(w, "node")?;
                    if !value.is_empty() {
                        write!(w, " [")?;
        
                        let mut iter = value.iter();
                        let first = iter.next().unwrap();
                        write!(w, "{}={}", first.0, first.1.to_dot_string())?;
                        for (key, value) in iter {
                            write!(w, ", ")?;
                            write!(w, "{}={}", key, value.to_dot_string())?;
                        }
                        write!(w, "]")?;
                    }
                    writeln!(w, ";")?;
                },
                AttributeType::None => {
                    if !value.is_empty() {        
                        for (key, value) in value.iter() {
                            writeln!(w, "{}={};", key, value.to_dot_string())?;
                        }
                    }
                }
            }
        }

        for n in self.graph.nodes {
            // TODO: handle render options
            // Are render options something we need?
            // we could clone the node or and remove the attributes based on render options
            // or maybe we keep a set of attributes to ignore based on the options
            writeln!(w, "{}", n.to_dot_string())?;
        }

        for e in self.graph.edges {
            write!(w, "{}", INDENT)?;
            write!(w, "{} {} {}", e.source, edge_op, e.target)?;
            // TODO: render ops
            if !e.attributes.is_empty() {
                write!(w, " [")?;

                let mut iter = e.attributes.iter();
                let first = iter.next().unwrap();
                write!(w, "{}={}", first.0, first.1.to_dot_string())?;
                for (key, value) in iter {
                    write!(w, ", ")?;
                    write!(w, "{}={}", key, value.to_dot_string())?;
                }
                write!(w, "]")?;
            }
            writeln!(w, ";")?;
        }

        writeln!(w, "}}")
    }
}

/// `Dot` configuration.
///
/// This enum does not have an exhaustive definition (will be expanded)
#[derive(Debug, PartialEq, Eq)]
pub enum Config {
    /// Use indices for node labels.
    NodeIndexLabel,
    /// Use indices for edge labels.
    EdgeIndexLabel,
    /// Use no edge labels.
    EdgeNoLabel,
    /// Use no node labels.
    NodeNoLabel,
    /// Do not print the graph/digraph string.
    GraphContentOnly,
    #[doc(hidden)]
    _Incomplete(()),
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Clone)]
pub enum AttributeType {
    Graph,
    Node,
    Edge,
    None
}

pub struct Graph<'a> {
    pub id: Option<String>,
    
    pub is_directed: bool,

    pub strict: bool,

    // Comment added to the first line of the source.
    pub comment: Option<String>,

    pub graph_attributes: Option<GraphAttributeStatement<'a>>,

    pub node_attributes: Option<NodeAttributeStatement<'a>>,

    // pub edge_attributes: IndexMap<String, AttributeText<'a>>,
    pub edge_attributes: Option<EdgeAttributeStatement<'a>>,

    pub attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,

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
        // edge_attributes: IndexMap<String, AttributeText<'a>>,
        edge_attributes: Option<EdgeAttributeStatement<'a>>,
        attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,
        nodes: Vec<Node<'a>>,
        edges: Vec<Edge<'a>>,
    ) -> Self {
        Graph {
            id,
            is_directed,
            strict,
            comment,
            graph_attributes,
            node_attributes,
            edge_attributes,
            attributes,
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

    // graph_attributes: IndexMap<String, AttributeText<'a>>,
    graph_attributes: Option<GraphAttributeStatement<'a>>,

    // node_attributes: IndexMap<String, AttributeText<'a>>,
    node_attributes: Option<NodeAttributeStatement<'a>>,

    // edge_attributes: IndexMap<String, AttributeText<'a>>,
    edge_attributes: Option<EdgeAttributeStatement<'a>>,

    attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,

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
            // edge_attributes: IndexMap::new(),
            attributes: IndexMap::new(),
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
            // edge_attributes: IndexMap::new(),
            edge_attributes: None,
            attributes: IndexMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            comment: None,
        }
    }

    // TODO: do we want a map for attribute list ie attributes not mapped to (graph/node/edge)

    pub fn add_graph_attributes(&mut self, graph_attributes: GraphAttributeStatement<'a>) -> &mut Self {
        self.graph_attributes = Some(graph_attributes);
        self
    }

    pub fn add_node_attributes(&mut self, node_attributes: NodeAttributeStatement<'a>) -> &mut Self {
        self.node_attributes = Some(node_attributes);
        self
    }

    // pub fn add_edge_attributes(&mut self, edge_attributes: IndexMap<String, AttributeText<'a>>) -> &mut Self {
    //     self.edge_attributes = edge_attributes;
    //     self
    // }
    pub fn add_edge_attributes(&mut self, edge_attributes: EdgeAttributeStatement<'a>) -> &mut Self {
        self.edge_attributes = Some(edge_attributes);
        self
    }

    // TODO: update to insert into appropriate statement or remove?
    pub fn add_attribute(
        &mut self, 
        attribute_type: AttributeType, 
        key: String, value: AttributeText<'a>
    ) -> &mut Self {
        self.get_attributes(attribute_type).insert(key, value);
        self
    }

    pub fn add_attributes(
        &mut self, 
        attribute_type: AttributeType, 
        attributes: HashMap<String, AttributeText<'a>>
    ) -> &mut Self {
        self.get_attributes(attribute_type).extend(attributes);
        self
    }

    fn get_attributes(&mut self, attribute_type: AttributeType) -> &mut IndexMap<String, AttributeText<'a>> {
        self.attributes.entry(attribute_type).or_insert(IndexMap::new())
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
            // edge_attributes: self.edge_attributes.clone(),
            edge_attributes: self.edge_attributes.clone(),
            attributes: self.attributes.clone(), // TODO: is clone the only option here?
            nodes: self.nodes.clone(), // TODO: is clone the only option here?
            edges: self.edges.clone(), // TODO: is clone the only option here?
        }
    }
}


trait GraphAttributes<'a> {

    fn background(&mut self, background: String) -> &mut Self {
        self.add_attribute("_background", AttributeText::attr(background))
    }

    // TODO: color list
    /// A colon-separated list of weighted color values: WC(:WC)* where each WC has the form C(;F)? 
    /// with C a color value and the optional F a floating-point number, 0 ≤ F ≤ 1. 
    /// The sum of the floating-point numbers in a colorList must sum to at most 1.
    fn background_color(&mut self, background_color: Color) -> &mut Self {
        self.add_attribute("bgcolor", AttributeText::quotted(background_color.to_dot_string()))
    }

    /// Type: rect which is "%f,%f,%f,%f"
    /// The rectangle llx,lly,urx,ury gives the coordinates, in points, of the lower-left corner (llx,lly) 
    /// and the upper-right corner (urx,ury).
    fn bounding_box(&mut self, bounding_box: String) -> &mut Self {
        self.add_attribute("bb", AttributeText::quotted(bounding_box))
    }

    /// If true, the drawing is centered in the output canvas.
    fn center(&mut self, center: bool) -> &mut Self {
        self.add_attribute("center", AttributeText::attr(center.to_string()))
    }

    /// Specifies the character encoding used when interpreting string input as a text label.
    fn charset(&mut self, charset: String) -> &mut Self {
        self.add_attribute("charset", AttributeText::quotted(charset))
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
        self.add_attribute("clusterrank", AttributeText::quotted(cluster_rank.as_slice()))
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
        self.add_attribute("compound", AttributeText::quotted(compound))
    }

    fn concentrate(&mut self, concentrate: String) -> &mut Self {
        self.add_attribute("concentrate", AttributeText::quotted(concentrate))
    }

    /// Specifies the expected number of pixels per inch on a display device.
    /// Also known as resolution
    fn dpi(&mut self, dpi: f32) -> &mut Self {
        self.add_attribute("dpi", AttributeText::quotted(dpi.to_string()))
    }

    // color list
    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    fn fill_color(&mut self, fill_color: Color) -> &mut Self {
        Attributes::fill_color(self.get_attributes_mut(), fill_color);
        self
    }

    // color list
    /// Color used for text.
    fn font_color(&mut self, font_color: Color) -> &mut Self {
        Attributes::font_color(self.get_attributes_mut(), font_color);
        self
    }

    /// Font used for text. 
    fn font_name(&mut self, font_name: String) -> &mut Self {
        Attributes::font_name(self.get_attributes_mut(), font_name);
        self
    }

    fn font_names(&mut self, font_names: String) -> &mut Self {
        self.add_attribute("fontnames", AttributeText::quotted(font_names))
    }
    
    fn font_path(&mut self, font_path: String) -> &mut Self {
        self.add_attribute("fontpath", AttributeText::quotted(font_path))
    }

    // TODO: constrain
    /// Font size, in points, used for text.
    /// default: 14.0, minimum: 1.0
    fn font_size(&mut self, font_size: f32) -> &mut Self {
        Attributes::font_size(self.get_attributes_mut(), font_size);
        self
    }

    fn force_label(&mut self, force_label: bool) -> &mut Self {
        self.add_attribute("forcelabel", AttributeText::attr(force_label.to_string()))
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

    fn label_scheme(&mut self, label_scheme: u32) -> &mut Self {
        self.add_attribute("labelscheme", AttributeText::attr(label_scheme.to_string()))
    }

    // If labeljust=r, the label is right-justified within bounding rectangle
    // If labeljust=l, left-justified
    // Else the label is centered.
    fn label_just(&mut self, label_just: LabelJustification) -> &mut Self {
        self.add_attribute("labeljust", AttributeText::attr(label_just.as_slice()))
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

    fn landscape(&mut self, landscape: bool) -> &mut Self {
        self.add_attribute("landscape", AttributeText::attr(landscape.to_string()))
    }

    /// Specifies the separator characters used to split an attribute of type layerRange into a list of ranges.
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
        self.add_attribute("lheight", AttributeText::attr(lheight.to_string()))
    }

    // TODO: I dont understand this...
    /// "%f,%f('!')?" representing the point (x,y). The optional '!' indicates the node position should not change (input-only).
    /// If dim=3, point may also have the format "%f,%f,%f('!')?" to represent the point (x,y,z).
    fn lp(&mut self, lp: String) -> &mut Self {
        Attributes::lp(self.get_attributes_mut(), lp);
        self
    }

    /// Width of graph or cluster label, in inches.
    fn lwidth(&mut self, lwidth: f32) -> &mut Self {
        self.add_attribute("lwidth", AttributeText::attr(lwidth.to_string()))
    }

    // TODO: point
    /// For graphs, this sets x and y margins of canvas, in inches.
    /// If the margin is a single double, both margins are set equal to the given value.
    /// Note that the margin is not part of the drawing but just empty space left around the drawing. 
    /// The margin basically corresponds to a translation of drawing, as would be necessary to center a drawing on a page. 
    /// Nothing is actually drawn in the margin. To actually extend the background of a drawing, see the pad attribute.
    /// For clusters, margin specifies the space between the nodes in the cluster and the cluster bounding box. By default, this is 8 points.
    /// For nodes, this attribute specifies space left around the node’s label. 
    /// By default, the value is 0.11,0.055.
    fn margin(&mut self, margin: f32) -> &mut Self {
        Attributes::margin(self.get_attributes_mut(), margin);
        self
    }

    /// Multiplicative scale factor used to alter the MinQuit (default = 8) and MaxIter (default = 24) parameters used during crossing minimization.
    /// These correspond to the number of tries without improvement before quitting and the maximum number of iterations in each pass.
    fn mclimit(&mut self, mclimit: f32) -> &mut Self {
        self.add_attribute("mclimit", AttributeText::attr(mclimit.to_string()))
    }

    /// Specifies the minimum separation between all nodes.
    fn mindist(&mut self, mindist: u32) -> &mut Self {
        self.add_attribute("mindist", AttributeText::attr(mindist.to_string()))
    }

    /// Whether to use a single global ranking, ignoring clusters.
    /// The original ranking algorithm in dot is recursive on clusters. 
    /// This can produce fewer ranks and a more compact layout, but sometimes at the cost of a head node being place on a higher rank than the tail node. 
    /// It also assumes that a node is not constrained in separate, incompatible subgraphs. 
    /// For example, a node cannot be in a cluster and also be constrained by rank=same with a node not in the cluster.
    /// This allows nodes to be subject to multiple constraints. 
    /// Rank constraints will usually take precedence over edge constraints.
    fn newrank(&mut self, newrank: bool) -> &mut Self {
        self.add_attribute("newrank", AttributeText::attr(newrank.to_string()))
    }

    // TODO: add constraint
    /// specifies the minimum space between two adjacent nodes in the same rank, in inches.
    /// default: 0.25, minimum: 0.02
    fn nodesep(&mut self, nodesep: f32) -> &mut Self {
        self.add_attribute("nodesep", AttributeText::attr(nodesep.to_string()))
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

    /// Sets number of iterations in network simplex applications.
    /// nslimit is used in computing node x coordinates.
    /// If defined, # iterations = nslimit * # nodes; otherwise, # iterations = MAXINT.
    fn nslimit(&mut self, nslimit: f32) -> &mut Self {
        self.add_attribute("nslimit", AttributeText::attr(nslimit.to_string()))
    }

    /// If ordering="out", then the outedges of a node, that is, edges with the node as its tail node, must appear left-to-right in the same order in which they are defined in the input.
    /// If ordering="in", then the inedges of a node must appear left-to-right in the same order in which they are defined in the input.
    /// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph or subgraph.
    /// Note that the graph attribute takes precedence over the node attribute.
    fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        self.add_attribute("ordering", AttributeText::attr(ordering.as_slice()))
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

    /// Specify order in which nodes and edges are drawn.
    /// default: breadthfirst
    fn output_order(&mut self, output_order: OutputMode) -> &mut Self {
        self.add_attribute("outputorder", AttributeText::attr(output_order.as_slice()))
    }

    /// Whether each connected component of the graph should be laid out separately, and then the graphs packed together.
    /// If false, the entire graph is laid out together. 
    /// The granularity and method of packing is influenced by the packmode attribute.
    fn pack(&mut self, pack: bool) -> &mut Self {
        self.add_attribute("pack", AttributeText::attr(pack.to_string()))
    }

    // TODO: constrain to non-negative integer.
    /// Whether each connected component of the graph should be laid out separately, and then the graphs packed together.
    /// This is used as the size, in points,of a margin around each part; otherwise, a default margin of 8 is used.
    /// pack is treated as true if the value of pack iso a non-negative integer.
    fn pack_int(&mut self, pack: u32) -> &mut Self {
        self.add_attribute("pack", AttributeText::attr(pack.to_string()))
    }

    /// This indicates how connected components should be packed (cf. packMode). 
    /// Note that defining packmode will automatically turn on packing as though one had set pack=true.
    fn pack_mode(&mut self, pack_mode: PackMode) -> &mut Self {
        self.add_attribute("packmode", AttributeText::attr(pack_mode.as_slice()))
    }

    /// Specifies how much, in inches, to extend the drawing area around the minimal area needed to draw the graph.
    /// Both the x and y pad values are set equal to the given value. 
    /// This area is part of the drawing and will be filled with the background color, if appropriate.
    /// default: 0.0555
    fn pad(&mut self, pad: f32) -> &mut Self {
        self.add_attribute("pad", AttributeText::attr(pad.to_string()))
    }

    /// Specifies how much, in inches, to extend the drawing area around the minimal area needed to draw the graph.
    fn pad_point(&mut self, pad: Point) -> &mut Self {
        self.add_attribute("pad", AttributeText::attr(pad.to_formatted_string()))
    }

    /// Width and height of output pages, in inches.
    /// Value given is used for both the width and height.
    fn page(&mut self, page: f32) -> &mut Self {
        self.add_attribute("page", AttributeText::attr(page.to_string()))
    }

    /// Width and height of output pages, in inches.
    fn page_point(&mut self, page: Point) -> &mut Self {
        self.add_attribute("page", AttributeText::attr(page.to_formatted_string()))
    }

    /// The order in which pages are emitted.
    /// Used only if page is set and applicable.
    /// Limited to one of the 8 row or column major orders.
    fn page_dir(&mut self, page_dir: PageDirection) -> &mut Self {
        self.add_attribute("pagedir", AttributeText::attr(page_dir.as_slice()))
    }

    // TODO: constrain
    /// If quantum > 0.0, node label dimensions will be rounded to integral multiples of the quantum.
    /// default: 0.0, minimum: 0.0
    fn quantum(&mut self, quantum: f32) -> &mut Self {
        self.add_attribute("quantum", AttributeText::attr(quantum.to_string()))
    }

    /// Sets direction of graph layout.
    /// For example, if rankdir="LR", and barring cycles, an edge T -> H; will go from left to right. 
    /// By default, graphs are laid out from top to bottom.
    /// This attribute also has a side-effect in determining how record nodes are interpreted. 
    /// See record shapes.
    fn rank_dir(&mut self, rank_dir: RankDir) -> &mut Self {
        self.add_attribute("rankdir", AttributeText::attr(rank_dir.as_slice()))
    }

    /// sets the desired rank separation, in inches.
    /// This is the minimum vertical distance between the bottom of the nodes in one rank 
    /// and the tops of nodes in the next. If the value contains equally, 
    /// the centers of all ranks are spaced equally apart. 
    /// Note that both settings are possible, e.g., ranksep="1.2 equally".
    fn rank_sep(&mut self, rank_sep: String) -> &mut Self {
        self.add_attribute("ranksep", AttributeText::attr(rank_sep))
    }

    // TODO: numeric vs string
    // Strings: fill, compress, expand, auto
    /// Sets the aspect ratio (drawing height/drawing width) for the drawing.
    /// Note that this is adjusted before the size attribute constraints are enforced.
    fn ratio(&mut self, ratio: String) -> &mut Self {
        self.add_attribute("ratio", AttributeText::attr(ratio))
    }

    /// If true and there are multiple clusters, run crossing minimization a second time.
    fn remincross(&mut self, remincross: bool) -> &mut Self {
        self.add_attribute("remincross", AttributeText::attr(remincross.to_string()))
    }

    /// If rotate=90, sets drawing orientation to landscape.
    fn rotate(&mut self, rotate: u32) -> &mut Self {
        self.add_attribute("rotate", AttributeText::attr(rotate.to_string()))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at the end if showboxes=2.
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
    fn size(&mut self, size: u32, desired_min: bool) -> &mut Self {
        let mut text = format!("{}", size);
        if desired_min {
            text.push_str("!");
        }
        self.add_attribute("size", AttributeText::attr(text))
    }

    /// Maximum width and height of drawing, in inches.
    /// If defined and the drawing is larger than the given size, the drawing 
    /// is uniformly scaled down so that it fits within the given size.
    /// If desired_min is true, and both both dimensions of the drawing 
    /// are less than size, the drawing is scaled up uniformly until at 
    /// least one dimension equals its dimension in size.
    fn size_point(&mut self, size: Point, desired_min: bool) -> &mut Self {
        let mut text = size.to_formatted_string();
        if desired_min {
            text.push_str("!");
        }
        self.add_attribute("size", AttributeText::attr(text))
    }

    /// If packmode indicates an array packing, sortv specifies an insertion order 
    /// among the components, with smaller values inserted first.
    /// default: 0, minimum: 0
    fn sortv(&mut self, sortv: u32) -> &mut Self {
        Attributes::sortv(self.get_attributes_mut(), sortv);
        self
    }

    /// Controls how, and if, edges are represented.
    /// If splines=true, edges are drawn as splines routed around nodes; 
    /// if splines=false, edges are drawn as line segments.
    fn splines_bool(&mut self, splines: bool) -> &mut Self {
        self.add_attribute("splines", AttributeText::attr(splines.to_string()))
    }

    /// Controls how, and if, edges are represented.
    fn splines(&mut self, splines: Splines) -> &mut Self {
        self.add_attribute("splines", AttributeText::attr(splines.as_slice()))
    }

    /// Set style information for components of the graph.
    fn style<S: Into<Cow<'a, str>>>(&mut self, style: S) -> &mut Self {
        Attributes::style(self.get_attributes_mut(), style);
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
    /// If truecolor is unset, truecolor is not used unless there is a shapefile property for some node in the graph. The output model will use the input model when possible.
    fn true_color(&mut self, true_color: bool) -> &mut Self {
        self.add_attribute("truecolor", AttributeText::attr(true_color.to_string()))
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
    /// The optional Z is the zoom factor, i.e., the image in the original layout will be W/Z by H/Z points in size. By default, Z is 1.
    /// The optional last part is either a pair (x,y) giving a position in the original layout of the graph, 
    /// in points, of the center of the viewport, or the name N of a node whose center should used as the focus.
    fn viewport(&mut self, viewport: String) -> &mut Self {
        self.add_attribute("viewport", AttributeText::attr(viewport))
    }

    /// Add an attribute to the node.
    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self;

    /// Add multiple attributes to the node.
    fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self;

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>>;

}

impl<'a> GraphAttributes<'a> for GraphAttributeStatementBuilder<'a> {

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>> {
        &mut self.attributes
    }

    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the node.
    fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
        self.attributes.extend(attributes);
        self
    }
}

// I'm not a huge fan of needing this builder but having a hard time getting around &mut without it
pub struct GraphAttributeStatementBuilder<'a> {
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> GraphAttributeStatementBuilder<'a>  {

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

    pub fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
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

pub enum ClusterMode {
    Local,
    Global,
    None
}

impl ClusterMode {
    pub fn as_slice(self) -> &'static str {
        match self {
            ClusterMode::Local => "local",
            ClusterMode::Global => "global",
            ClusterMode::None => "none",
        }
    }
}

pub enum LabelJustification {
    Left,
    Right,
    Center
}

impl LabelJustification {
    pub fn as_slice(self) -> &'static str {
        match self {
            LabelJustification::Left => "l",
            LabelJustification::Right => "r",
            LabelJustification::Center => "c",
        }
    }
}


pub enum LabelLocation {
    Top,
    Center,
    Bottom
}

impl LabelLocation {
    pub fn as_slice(self) -> &'static str {
        match self {
            LabelLocation::Top => "t",
            LabelLocation::Center => "c",
            LabelLocation::Bottom => "b",
        }
    }
}

pub enum Ordering {
    In,
    Out,
}

impl Ordering {
    pub fn as_slice(self) -> &'static str {
        match self {
            Ordering::In => "in",
            Ordering::Out => "out",
        }
    }
}

/// These specify the order in which nodes and edges are drawn in concrete output.
/// The default "breadthfirst" is the simplest, but when the graph layout does not avoid edge-node overlap, 
/// this mode will sometimes have edges drawn over nodes and sometimes on top of nodes.
/// If the mode "nodesfirst" is chosen, all nodes are drawn first, followed by the edges. 
/// This guarantees an edge-node overlap will not be mistaken for an edge ending at a node.
/// On the other hand, usually for aesthetic reasons, it may be desirable that all edges appear beneath nodes, 
/// even if the resulting drawing is ambiguous. 
/// This can be achieved by choosing "edgesfirst".
pub enum OutputMode {
    BreadthFirst,
    NodesFirst,
    EdgesFirst,
}

impl OutputMode {
    pub fn as_slice(self) -> &'static str {
        match self {
            OutputMode::BreadthFirst => "breadthfirst",
            OutputMode::NodesFirst => "nodesfirst",
            OutputMode::EdgesFirst => "edgesfirst",
        }
    }
}

/// The modes "node", "clust" or "graph" specify that the components should be packed together tightly, 
/// using the specified granularity. 
pub enum PackMode {
    /// causes packing at the node and edge level, with no overlapping of these objects.
    /// This produces a layout with the least area, but it also allows interleaving, 
    /// where a node of one component may lie between two nodes in another component.
    Node,

    /// guarantees that top-level clusters are kept intact. 
    /// What effect a value has also depends on the layout algorithm. 
    Cluster,

    /// does a packing using the bounding box of the component. 
    /// Thus, there will be a rectangular region around a component free of elements of any other component. 
    Graph,
    // TODO: array - "array(_flags)?(%d)?"
}

impl PackMode {
    pub fn as_slice(self) -> &'static str {
        match self {
            PackMode::Node => "node",
            PackMode::Cluster => "clust",
            PackMode::Graph => "graph",
        }
    }
}

// The optional '!' indicates the node position should not change (input-only).
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: Option<f32>,

    /// specify that the node position should not change.
    pub force_pos: bool,
}

impl Point {

    pub fn new_2d(x: f32, y: f32, force_pos: bool) -> Self {
        Self {
            x,
            y,
            z: None,
            force_pos,
        }
    }

    pub fn new_3d(x: f32, y: f32, z: f32, force_pos: bool) -> Self {
        Self {
            x,
            y,
            z: Some(z),
            force_pos,
        }
    }

    // TODO: change to to_dot_string / print_dot
    pub fn to_formatted_string(self) -> String { 
        let mut slice = format!("{},{}", self.x, self.y);
        if self.z.is_some() {
            slice.push_str(format!(",{}", self.z.unwrap()).as_str());
        }
        if !self.force_pos {
            slice.push_str("!")
        }
        slice.to_string()
    }
}

pub enum PageDirection {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
    RightBottom,
    RightTop,
    LeftBottom,
    LeftTop,
}

impl PageDirection {
    pub fn as_slice(self) -> &'static str {
        match self {
            PageDirection::BottomLeft => "BL",
            PageDirection::BottomRight => "BR",
            PageDirection::TopLeft => "TL",
            PageDirection::TopRight => "TR",
            PageDirection::RightBottom => "RB",
            PageDirection::RightTop => "RT",
            PageDirection::LeftBottom => "LB",
            PageDirection::LeftTop => "LT",
        }
    }
}

pub enum RankDir {
    TopBottom,
    LeftRight,
    BottomTop,
    RightLeft,
}

impl RankDir {
    pub fn as_slice(self) -> &'static str {
        match self {
            RankDir::TopBottom => "TB",
            RankDir::LeftRight => "LR",
            RankDir::BottomTop => "BT",
            RankDir::RightLeft => "RL",
        }
    }
}

pub enum Splines {
    Line,
    Spline,
    None,
    Curved,
    Polyline,
    Ortho,
}

impl Splines {
    pub fn as_slice(self) -> &'static str {
        match self {
            Splines::Line => "line",
            Splines::Spline => "spline",
            Splines::None => "none",
            Splines::Curved => "curved",
            Splines::Polyline => "polyline",
            Splines::Ortho => "ortho",
        }
    }
}

/// The number of points in the list must be equivalent to 1 mod 3; note that this is not checked.
pub struct SplineType {
    start: Option<Point>,
    end: Option<Point>,
    spline_points: Vec<Point>,
}

impl SplineType {

    // TODO: is this implementation correct?
    pub fn to_dot_string(self) -> String { 
        let mut dot_string = "".to_owned();

        if let Some(end) = self.end {
            dot_string.push_str(format!("e,{},{} ", end.x, end.y).as_str())
        }

        if let Some(start) = self.start {
            dot_string.push_str(format!("s,{},{} ", start.x, start.y).as_str())
        }

        for s in self.spline_points {
            dot_string.push_str(format!("{}", s.to_formatted_string()).as_str())
        }

        dot_string.to_string()
    }

}


// TODO: add node builder using "with" convention
#[derive(Clone, Debug)]
pub struct Node<'a> {

    pub id: String,
    pub port: Option<String>,
    // pub compass: Option<Compass>,
    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> Node<'a> {

    pub fn new(id: String) -> Node<'a> {
        // TODO: constrain id
        Node {
            id,
            port: None,
            attributes: IndexMap::new(),
        }
    }

    /// Set the port for the node.
    pub fn port(mut self, port: String) -> Self {
        self.port = Some(port);
        self
    }

    // pub fn compass<'a>(&'a mut self, compass: Compass) -> &'a mut Node {
    //     self.compass = Some(compass);
    //     self
    // }

    pub fn to_dot_string(&self) -> String {
        let mut dot_string = format!("{}{}", INDENT, &self.id);
        // TODO: I dont love this logic. I would like to find away to not have a special case.
        // I think we introduce a AttributeText enum which encodes how to write out the attribute value
        if !self.attributes.is_empty() {
            dot_string.push_str(" [");
            let mut iter = self.attributes.iter();
            let first = iter.next().unwrap();
            dot_string.push_str(format!("{}={}", first.0, first.1.to_dot_string()).as_str());
            for (key, value) in iter {
                dot_string.push_str(", ");
                dot_string.push_str(format!("{}={}", key, value.to_dot_string()).as_str());
            }

            dot_string.push_str("]");
        }
        dot_string.push_str(";");
        return dot_string.to_string();
    }
}


pub struct NodeBuilder<'a> {
    id: String,

    port: Option<String>,
    
    attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> NodeAttributes<'a> for NodeBuilder<'a> {
    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the edge.
    fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
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
            port: None,
            attributes: IndexMap::new(),
        }
    }

    /// Set the port for the node.
    pub fn port<S: Into<String>>(&mut self, port: S) -> &mut Self {
        self.port = Some(port.into());
        self
    }

    pub fn build(&self) -> Node<'a> {
        Node {
            // TODO: are these to_owned and clones necessary?
            id: self.id.to_owned(),
            port: self.port.to_owned(),
            attributes: self.attributes.clone()
        }
    }
}

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

impl ImagePosition {
    pub fn as_slice(self) -> &'static str {
        match self {
            ImagePosition::TopLeft => "tl",
            ImagePosition::TopCentered => "tc",
            ImagePosition::TopRight => "tr",
            ImagePosition::MiddleLeft => "ml",
            ImagePosition::MiddleCentered => "mc",
            ImagePosition::MiddleRight => "mr",
            ImagePosition::BottomLeft => "bl",
            ImagePosition::BottomCentered => "bc",
            ImagePosition::BottomRight => "br",
        }
    }
}

pub enum ImageScale {
    Width,
    Height,
    Both,
}

impl ImageScale {
    pub fn as_slice(self) -> &'static str {
        match self {
            ImageScale::Width => "width",
            ImageScale::Height => "height",
            ImageScale::Both => "both",
        }
    }
}


#[derive(Clone, Debug)]
pub struct Edge<'a> {

    pub source: String,

    pub target: String,

    pub attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> Edge<'a> {

    pub fn new(source: String, target: String) -> Edge<'a> {
        Edge {
            source,
            target,
            attributes: IndexMap::new(),
        }
    }
}

pub struct EdgeBuilder<'a> {
    pub source: String,

    pub target: String,
    
    attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> EdgeAttributes<'a> for EdgeBuilder<'a> {
    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>> {
        &mut self.attributes
    }

    // /// Add multiple attribures to the edge.
    // fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
    //     self.attributes.extend(attributes);
    //     self
    // }
}

impl<'a> EdgeBuilder<'a> {
    pub fn new(source: String, target: String) -> EdgeBuilder<'a> {
        EdgeBuilder {
            source,
            target,
            attributes: IndexMap::new(),
        }
    }

    /// Add an attribute to the edge.
    pub fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the edge.
    pub fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
        self.attributes.extend(attributes);
        self
    }

    pub fn build(&self) -> Edge<'a> {
        Edge {
            // TODO: are these to_owned and clones necessary?
            source: self.source.to_owned(),
            target: self.target.to_owned(),
            attributes: self.attributes.clone()
        }
    }
}

trait AttributeStatement<'a> {
    fn get_attribute_statement_type(&self) -> &'static str;

    fn get_attributes(&self) -> &IndexMap<String, AttributeText<'a>>;

    fn to_dot_string(&self) -> String {
        if self.get_attributes().is_empty() {
            return String::from("");
        }
        let mut dot_string = format!("{}{} [", INDENT, self.get_attribute_statement_type());
        let attributes = &self.get_attributes();
        let mut iter = attributes.iter();
        let first = iter.next().unwrap();
        dot_string.push_str(format!("{}={}", first.0, first.1.to_dot_string()).as_str());
        for (key, value) in iter {
            dot_string.push_str(", ");
            dot_string.push_str(format!("{}={}", key, value.to_dot_string()).as_str());
        }
        dot_string.push_str("];\n");
        dot_string.to_string()
    }
}

trait NodeAttributes<'a> {

    // TODO: constrain
    /// Indicates the preferred area for a node or empty cluster when laid out by patchwork.
    /// default: 1.0, minimum: >0
    fn area(&mut self, area: f32) -> &mut Self {
        self.add_attribute("area", AttributeText::attr(area.to_string()))
    }

    /// Classnames to attach to the node’s SVG element. 
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    fn class(&mut self, class: String) -> &mut Self {
        Attributes::class(self.get_attributes_mut(), class);
        self
    }

    // color / color list
    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    fn color(&mut self, color: Color) -> &mut Self {
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
        self.add_attribute("distortion", AttributeText::attr(distortion.to_string()))
    }

    // color list
    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    fn fill_color(&mut self, fill_color: Color) -> &mut Self {
        Attributes::fill_color(self.get_attributes_mut(), fill_color);
        self
    }

    /// If true, the node size is specified by the values of the width and height attributes only and 
    /// is not expanded to contain the text label. 
    /// There will be a warning if the label (with margin) cannot fit within these limits.
    /// If false, the size of a node is determined by smallest width and height needed to contain its label 
    /// and image, if any, with a margin specified by the margin attribute.
    fn fixed_size(&mut self, fixed_size: bool) -> &mut Self {
        self.add_attribute("fixedsize", AttributeText::quotted(fixed_size.to_string()))
    }

    // TODO: color list
    /// Color used for text.
    fn font_color(&mut self, font_color: Color) -> &mut Self {
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
        self.add_attribute("height", AttributeText::attr(height.to_string()))
    }

    // TODO: delete and just use url?
    /// Synonym for URL.
    fn href(&mut self, href: String) -> &mut Self {
        self.add_attribute("href", AttributeText::escaped(href))
    }

    /// Gives the name of a file containing an image to be displayed inside a node. 
    /// The image file must be in one of the recognized formats, 
    /// typically JPEG, PNG, GIF, BMP, SVG, or Postscript, and be able to be converted 
    /// into the desired output format.
    fn image(&mut self, image: String) -> &mut Self {
        self.add_attribute("image", AttributeText::quotted(image))
    }

    /// Controls how an image is positioned within its containing node.
    /// Only has an effect when the image is smaller than the containing node.
    fn image_pos(&mut self, image_pos: ImagePosition) -> &mut Self {
        self.add_attribute("imagepos", AttributeText::quotted(image_pos.as_slice()))
    }

    /// Controls how an image fills its containing node.
    fn image_scale_bool(&mut self, image_scale: bool) -> &mut Self {
        self.add_attribute("imagescale", AttributeText::quotted(image_scale.to_string()))
    }

    /// Controls how an image fills its containing node.
    fn image_scale(&mut self, image_scale: ImageScale) -> &mut Self {
        self.add_attribute("imagescale", AttributeText::quotted(image_scale.as_slice()))
    }

    /// Text label attached to objects.
    fn label<S: Into<Cow<'a, str>>>(&mut self, text: S) -> &mut Self {
        self.add_attribute("label", AttributeText::quotted(text))
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

    // TODO: point
    /// For nodes, this attribute specifies space left around the node’s label.
    /// If the margin is a single double, both margins are set equal to the given value.
    /// Note that the margin is not part of the drawing but just empty space left around the drawing. 
    /// The margin basically corresponds to a translation of drawing, as would be necessary to center a drawing on a page. 
    /// Nothing is actually drawn in the margin. To actually extend the background of a drawing, see the pad attribute.
    /// By default, the value is 0.11,0.055.
    fn margin(&mut self, margin: f32) -> &mut Self {
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

    /// If ordering="out", then the outedges of a node, that is, edges with the node as its tail node, must appear left-to-right in the same order in which they are defined in the input.
    /// If ordering="in", then the inedges of a node must appear left-to-right in the same order in which they are defined in the input.
    /// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph or subgraph.
    /// Note that the graph attribute takes precedence over the node attribute.
    fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        self.add_attribute("ordering", AttributeText::attr(ordering.as_slice()))
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
        self.add_attribute("penwidth", AttributeText::attr(peripheries.to_string()))
    }

    /// Position of node, or spline control points.
    /// the position indicates the center of the node. On output, the coordinates are in points.
    fn pos(&mut self, pos: Point) -> &mut Self {
        Attributes::pos(self.get_attributes_mut(), pos);
        self
    }

    // TODO: add post_spline

    // TODO: add rect type?
    // "%f,%f,%f,%f"
    /// Rectangles for fields of records, in points.
    fn rects(&mut self, rects: String) -> &mut Self {
        self.add_attribute("rects", AttributeText::attr(rects))
    }

    /// If true, force polygon to be regular, i.e., the vertices of the polygon will 
    /// lie on a circle whose center is the center of the node.
    fn regular(&mut self, regular: bool) -> &mut Self {
        self.add_attribute("regular", AttributeText::attr(regular.to_string()))
    }

    /// Gives the number of points used for a circle/ellipse node.
    fn sample_points(&mut self, sample_points: u32) -> &mut Self {
        self.add_attribute("samplepoints", AttributeText::attr(sample_points.to_string()))
    }

    /// Sets the shape of a node.
    fn shape(&mut self, shape: Shape) -> &mut Self {
        self.add_attribute("shape", AttributeText::attr(shape.as_slice()))
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
        self.add_attribute("sides", AttributeText::attr(sides.to_string()))
    }

    // TODO: constrain
    /// Skew factor for shape=polygon.
    /// Positive values skew top of polygon to right; negative to left.
    /// default: 0.0, minimum: -100.0
    fn skew(&mut self, skew: f32) -> &mut Self {
        self.add_attribute("skew", AttributeText::attr(skew.to_string()))
    }

    /// If packmode indicates an array packing, sortv specifies an insertion order 
    /// among the components, with smaller values inserted first.
    /// default: 0, minimum: 0
    fn sortv(&mut self, sortv: u32) -> &mut Self {
        Attributes::sortv(self.get_attributes_mut(), sortv);
        self
    }

    /// Set style information for components of the graph.
    fn style<S: Into<Cow<'a, str>>>(&mut self, style: S) -> &mut Self {
        Attributes::style(self.get_attributes_mut(), style);
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
        self.add_attribute("vertices", AttributeText::quotted(vertices))
    }

    /// Width of node, in inches.
    /// This is taken as the initial, minimum width of the node. 
    /// If fixedsize is true, this will be the final width of the node. 
    /// Otherwise, if the node label requires more width to fit, the node’s 
    /// width will be increased to contain the label.
    fn width(&mut self, width: f32) -> &mut Self {
        self.add_attribute("width", AttributeText::attr(width.to_string()))
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
    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self;

    /// Add multiple attribures to the node.
    fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self;

    fn get_attributes_mut(&mut self) -> &mut IndexMap<String, AttributeText<'a>>;

}

impl<'a> NodeAttributes<'a> for NodeAttributeStatementBuilder<'a> {

    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the node.
    fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
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

impl<'a> NodeAttributeStatementBuilder<'a>  {

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

    pub fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
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


trait EdgeAttributes<'a> {

    /// Style of arrowhead on the head node of an edge. 
    /// This will only appear if the dir attribute is forward or both.
    fn arrow_head(&mut self, arrowhead: ArrowType) -> &mut Self {
        self.add_attribute("arrowhead", AttributeText::attr(arrowhead.as_slice()))
    }

    // TODO: constrain
    /// Multiplicative scale factor for arrowheads.
    /// default: 1.0, minimum: 0.0
    fn arrow_size(&mut self, arrow_size: f32) -> &mut Self {
        self.add_attribute("arrowsize", AttributeText::attr(arrow_size.to_string()))
    }

    /// Style of arrowhead on the tail node of an edge. 
    /// This will only appear if the dir attribute is back or both.
    fn arrowtail(&mut self, arrowtail: ArrowType) -> &mut Self {
        self.add_attribute("arrowtail", AttributeText::attr(arrowtail.as_slice()))
    }
    
    /// Classnames to attach to the edge’s SVG element. 
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    fn class(&mut self, class: String) -> &mut Self {
        Attributes::class(self.get_attributes_mut(), class);
        self
    }

    // color / color list
    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    fn color(&mut self, color: Color) -> &mut Self {
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
        self.add_attribute("comment", AttributeText::attr(comment.to_string()));
        self
    }

    /// If false, the edge is not used in ranking the nodes.
    fn constriant(&mut self, constriant: bool) -> &mut Self {
        self.add_attribute("constriant", AttributeText::attr(constriant.to_string()))
    }

    /// If true, attach edge label to edge by a 2-segment polyline, underlining the label, 
    /// then going to the closest point of spline.
    fn decorate(&mut self, decorate: bool) -> &mut Self {
        self.add_attribute("decorate", AttributeText::attr(decorate.to_string()))
    }

    /// Edge type for drawing arrowheads.
    /// Indicates which ends of the edge should be decorated with an arrowhead.
    /// The actual style of the arrowhead can be specified using the arrowhead and arrowtail attributes.
    fn dir(&mut self, dir: Direction) -> &mut Self {
        self.add_attribute("dir", AttributeText::attr(dir.as_slice()))
    }

    /// If the edge has a URL or edgeURL attribute, edgetarget determines which window 
    /// of the browser is used for the URL attached to the non-label part of the edge.
    /// Setting edgetarget=_graphviz will open a new window if it doesn’t already exist, or reuse it if it does.
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

    // color list
    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    fn fill_color(&mut self, fill_color: Color) -> &mut Self {
        Attributes::fill_color(self.get_attributes_mut(), fill_color);
        self
    }

    // color list
    /// Color used for text.
    fn font_color(&mut self, font_color: Color) -> &mut Self {
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
        self.add_attribute("head_lp", AttributeText::quotted(head_lp.to_formatted_string()))
    }

    /// If true, the head of an edge is clipped to the boundary of the head node; 
    /// otherwise, the end of the edge goes to the center of the node, or the center 
    /// of a port, if applicable.
    fn head_clip(&mut self, head_clip: bool) -> &mut Self {
        self.add_attribute("headclip", AttributeText::quotted(head_clip.to_string()))
    }

    /// Text label to be placed near head of edge.
    fn head_label(&mut self, head_label: String) -> &mut Self {
        self.add_attribute("headlabel", AttributeText::quotted(head_label))
    }

    // TODO: portPos struct?
    /// Indicates where on the head node to attach the head of the edge. 
    /// In the default case, the edge is aimed towards the center of the node, 
    /// and then clipped at the node boundary.
    fn head_port(&mut self, head_port: String) -> &mut Self {
        self.add_attribute("headport", AttributeText::quotted(head_port))
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
        self.add_attribute("labelangle", AttributeText::attr(label_angle.to_string()))
    }

    /// Multiplicative scaling factor adjusting the distance that the headlabel / taillabel is from the head / tail node.
    /// default: 1.0, minimum: 0.0
    fn label_distance(&mut self, label_distance: f32) -> &mut Self {
        self.add_attribute("labeldistance", AttributeText::attr(label_distance.to_string()))
    }

    /// If true, allows edge labels to be less constrained in position. 
    /// In particular, it may appear on top of other edges.
    fn label_float(&mut self, label_float: bool) -> &mut Self {
        self.add_attribute("labelfloat", AttributeText::attr(label_float.to_string()))
    }

    /// Color used for headlabel and taillabel.
    fn label_font_color(&mut self, label_font_color: Color) -> &mut Self {
        self.add_attribute("labelfontcolor", AttributeText::quotted(label_font_color.to_dot_string()))
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
        self.add_attribute("labelfontsize", AttributeText::attr(label_font_size.to_string()))
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
        self.add_attribute("lhead", AttributeText::quotted(lhead))
    }

    fn lp(&mut self, lp: String) -> &mut Self {
        Attributes::lp(self.get_attributes_mut(), lp);
        self
    }

    /// Logical tail of an edge.
    /// When compound=true, if ltail is defined and is the name of a cluster 
    /// containing the real tail, the edge is clipped to the boundary of the cluster.
    fn ltail(&mut self, ltail: String) -> &mut Self {
        self.add_attribute("ltail", AttributeText::quotted(ltail))
    }

    /// Minimum edge length (rank difference between head and tail).
    fn min_len(&mut self, min_len: u32) -> &mut Self {
        self.add_attribute("minlen", AttributeText::attr(min_len.to_string()))
    }

    fn no_justify(&mut self, no_justify: bool) -> &mut Self {
        self.add_attribute("nojustify", AttributeText::attr(no_justify.to_string()))
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
        self.add_attribute("samehead", AttributeText::quotted(same_head))
    }

    /// Edges with the same tail and the same sametail value are aimed at the same point on the tail.
    fn same_tail(&mut self, same_tail: String) -> &mut Self {
        self.add_attribute("sametail", AttributeText::quotted(same_tail))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at the end if showboxes=2.
    /// (Debugging, TB mode only!)
    /// default: 0, minimum: 0
    fn show_boxes(&mut self, show_boxes: u32) -> &mut Self {
        Attributes::show_boxes(self.get_attributes_mut(), show_boxes);
        self
    }

    /// Set style information for components of the graph.
    fn style<S: Into<Cow<'a, str>>>(&mut self, style: S) -> &mut Self {
        Attributes::style(self.get_attributes_mut(), style);
        self
    }

    /// Position of an edge’s tail label, in points.
    /// The position indicates the center of the label.
    fn tail_lp(&mut self, tail_lp: Point) -> &mut Self {
        self.add_attribute("tail_lp", AttributeText::quotted(tail_lp.to_formatted_string()))
    }

    /// If true, the tail of an edge is clipped to the boundary of the tail node; otherwise, 
    /// the end of the edge goes to the center of the node, or the center of a port, if applicable.
    fn tail_clip(&mut self, tail_clip: bool) -> &mut Self {
        self.add_attribute("tailclip", AttributeText::quotted(tail_clip.to_string()))
    }

    /// Text label to be placed near tail of edge.
    fn tail_label(&mut self, tail_label: String) -> &mut Self {
        self.add_attribute("taillabel", AttributeText::quotted(tail_label))
    }

    // TODO: portPos struct?
    /// Indicates where on the tail node to attach the tail of the edge.
    fn tail_port(&mut self, tail_port: String) -> &mut Self {
        self.add_attribute("tailport", AttributeText::quotted(tail_port))
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

    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self;

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

    fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
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

impl<'a> EdgeAttributeStatementBuilder<'a>  {

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

    pub fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

pub enum Shape {
    Box,
    Polygon,
    Ellipse,
    Oval,
    Circle,
    Point,
    Egg,
    Triangle,
    Plaintext,
    Plain,
    Diamond,
    Trapezium,
    Parallelogram,
    House,
    Pentagon,
    Hexagon,
    Septagon,
    Octagon,
    DoubleCircle,
    DoubleOctagon,
    TripleOctagon,
    Invtriangle,
    Invtrapezium,
    Invhouse,
    Mdiamond,
    Msquare,
    Mcircle,
    Rect,
    Rectangle,
    Square,
    Star,
    None,
    Underline,
    Cylinder,
    Note,
    Tab,
    Folder,
    Box3D,
    Component,
    Promoter,
    Cds,
    Terminator,
    Utr,
    Primersite,
    Restrictionsite,
    FivePoverHang,
    ThreePoverHang,
    NoverHang,
    Assemply,
    Signature,
    Insulator,
    Ribosite,
    Rnastab,
    Proteasesite,
    Proteinstab,
    Rpromotor,
    Rarrow,
    Larrow,
    Lpromotor,
}

impl Shape {
    pub fn as_slice(self) -> &'static str {
        match self {
            Shape::Box => "box",
            Shape::Polygon => "polygon",
            Shape::Ellipse => "ellipse",
            Shape::Oval => "oval",
            Shape::Circle => "circle",
            Shape::Point => "point",
            Shape::Egg => "egg",
            Shape::Triangle => "triangle",
            Shape::Plaintext => "plaintext",
            Shape::Plain => "plain",
            Shape::Diamond => "diamond",
            Shape::Trapezium => "trapezium",
            Shape::Parallelogram => "parallelogram",
            Shape::House => "house",
            Shape::Pentagon => "pentagon",
            Shape::Hexagon => "hexagon",
            Shape::Septagon => "septagon",
            Shape::Octagon => "octagon",
            Shape::DoubleCircle => "doublecircle",
            Shape::DoubleOctagon => "doubleoctagon",
            Shape::TripleOctagon => "tripleocctagon",
            Shape::Invtriangle => "invtriangle",
            Shape::Invtrapezium => "invtrapezium",
            Shape::Invhouse => "invhouse",
            Shape::Mdiamond => "mdiamond",
            Shape::Msquare => "msquare",
            Shape::Mcircle => "mcircle",
            Shape::Rect => "rect",
            Shape::Rectangle => "rectangle",
            Shape::Square => "square",
            Shape::Star => "star",
            Shape::None => "none",
            Shape::Underline => "underline",
            Shape::Cylinder => "cylinder",
            Shape::Note => "note",
            Shape::Tab => "tab",
            Shape::Folder => "folder",
            Shape::Box3D => "box3d",
            Shape::Component => "component",
            Shape::Promoter => "promoter",
            Shape::Cds => "cds",
            Shape::Terminator => "terminator",
            Shape::Utr => "utr",
            Shape::Primersite => "primersite",
            Shape::Restrictionsite => "restrictionsite",
            Shape::FivePoverHang => "fivepoverhang",
            Shape::ThreePoverHang => "threepoverhang",
            Shape::NoverHang => "noverhang",
            Shape::Assemply => "assemply",
            Shape::Signature => "signature",
            Shape::Insulator => "insulator",
            Shape::Ribosite => "ribosite",
            Shape::Rnastab => "rnastab",
            Shape::Proteasesite => "proteasesite",
            Shape::Proteinstab => "proteinstab",
            Shape::Rpromotor => "rpromotor",
            Shape::Rarrow => "rarrow",
            Shape::Larrow => "larrow",
            Shape::Lpromotor => "lpromotor",
        }
    }
}


pub enum ArrowType {
    Normal,
    Dot,
    Odot,
    None,
    Empty,
    Diamond,
    Ediamond,
    Box,
    Open,
    Vee,
    Inv,
    Invdot,
    Invodot,
    Tee,
    Invempty,
    Odiamond,
    Crow,
    Obox,
    Halfopen,
}

impl ArrowType {
    pub fn as_slice(self) -> &'static str {
        match self {
            ArrowType::Normal => "normal",
            ArrowType::Dot => "dot",
            ArrowType::Odot => "odot",
            ArrowType::None => "none",
            ArrowType::Empty => "empty",
            ArrowType::Diamond => "diamond",
            ArrowType::Ediamond => "ediamond",
            ArrowType::Box => "box",
            ArrowType::Open => "open",
            ArrowType::Vee => "vee",
            ArrowType::Inv => "inv",
            ArrowType::Invdot => "invdot",
            ArrowType::Invodot => "invodot",
            ArrowType::Tee => "tee",
            ArrowType::Invempty => "invempty",
            ArrowType::Odiamond => "odiamond",
            ArrowType::Crow => "crow",
            ArrowType::Obox => "obox",
            ArrowType::Halfopen => "halfopen",
        }
    }
}

pub enum Direction {
    Forward,
    Back,
    Both,
    None,
}

impl Direction {
    pub fn as_slice(self) -> &'static str {
        match self {
            Direction::Forward => "forward",
            Direction::Back => "back",
            Direction::Both => "both",
            Direction::None => "none",
        }
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

// TODO: is this a good representation of color?
pub enum Color<'a> {
    RGB {
        red: u8,
        green: u8,
        blue: u8
    },
    RGBA {
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    },
    // Hue-Saturation-Value (HSV) 0.0 <= H,S,V <= 1.0
    HSV {
        hue: f32,
        saturation: f32,
        value: f32
    },
    X11(X11<'a>),
}

impl<'a> Color<'a> {
 
    pub fn to_dot_string(self) -> String { 
        match self {
            Color::RGB { red, green, blue } => {
                format!("#{}{}{}", red, green, blue)
            },
            Color::RGBA { red, green, blue, alpha } => {
                format!("#{}{}{}{}", red, green, blue, alpha)
            }, 
            Color::HSV { hue, saturation, value } => {
                format!("{} {} {}", hue, saturation, value)
            },
            Color::X11(color) => {
                color.name.to_string()
            }
        }
    }
}

// TODO: is this better then just using str?
pub struct X11<'a> {
    name: Cow<'a, str>
}

impl<'a> X11<'a> {
    pub fn new<S>(name: S) -> X11<'a>
        where S: Into<Cow<'a, str>>
    {
        X11 { name: name.into() }
    }
}


/// The style for a node or edge.
/// See <http://www.graphviz.org/doc/info/attrs.html#k:style> for descriptions.
/// Note that some of these are not valid for edges.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Style {
    None,
    Solid,
    Dashed,
    Dotted,
    Bold,
    Rounded,
    Diagonals,
    Filled,
    Striped,
    Wedged,
}

impl Style {
    pub fn as_slice(self) -> &'static str {
        match self {
            Style::None => "",
            Style::Solid => "solid",
            Style::Dashed => "dashed",
            Style::Dotted => "dotted",
            Style::Bold => "bold",
            Style::Rounded => "rounded",
            Style::Diagonals => "diagonals",
            Style::Filled => "filled",
            Style::Striped => "striped",
            Style::Wedged => "wedged",
        }
    }
}

struct Attributes;

impl Attributes {
    fn class(attributes: &mut IndexMap<String, AttributeText>, class: String) {
        Self::add_attribute(attributes, "class", AttributeText::quotted(class))
    }

    fn color<'a>(attributes: &mut IndexMap<String, AttributeText>, color: Color<'a>) {
        Self::add_attribute(attributes,"color", AttributeText::quotted(color.to_dot_string()))
    }
    
    fn color_scheme(attributes: &mut IndexMap<String, AttributeText>, color_scheme: String) {
        Self::add_attribute(attributes, "colorscheme", AttributeText::quotted(color_scheme))
    }
    
    fn comment(attributes: &mut IndexMap<String, AttributeText>, comment: String) {
        Self::add_attribute(attributes, "comment", AttributeText::quotted(comment))
    }

    fn fill_color(attributes: &mut IndexMap<String, AttributeText>, fill_color: Color) {
        Self::add_attribute(attributes, "fillcolor", AttributeText::quotted(fill_color.to_dot_string()))
    }

    fn font_color(attributes: &mut IndexMap<String, AttributeText>, font_color: Color) {
        Self::add_attribute(attributes, "fontcolor", AttributeText::quotted(font_color.to_dot_string()))
    }

    fn font_name(attributes: &mut IndexMap<String, AttributeText>, font_name: String) {
        Self::add_attribute(attributes, "fontname", AttributeText::quotted(font_name))
    }

    fn font_size(attributes: &mut IndexMap<String, AttributeText>, font_size: f32) {
        Self::add_attribute(attributes, "fontsize", AttributeText::attr(font_size.to_string()))
    }
    
    fn gradient_angle(attributes: &mut IndexMap<String, AttributeText>, gradient_angle: u32) {
        Self::add_attribute(attributes, "gradientangle", AttributeText::attr(gradient_angle.to_string()))
    }

    fn label(attributes: &mut IndexMap<String, AttributeText>, text: String) {
        Self::add_attribute(attributes, "label", AttributeText::quotted(text));
    }

    fn label_location(attributes: &mut IndexMap<String, AttributeText>, label_location: LabelLocation) {
        Self::add_attribute(attributes, "labelloc", AttributeText::attr(label_location.as_slice()))
    }

    // TODO: layer struct
    fn layer(attributes: &mut IndexMap<String, AttributeText>, layer: String) {
        Self::add_attribute(attributes, "layer", AttributeText::attr(layer))
    }

    fn lp(attributes: &mut IndexMap<String, AttributeText>, lp: String) {
        Self::add_attribute(attributes, "lp", AttributeText::attr(lp))
    }

    fn margin(attributes: &mut IndexMap<String, AttributeText>, margin: f32) {
        Self::add_attribute(attributes, "margin", AttributeText::attr(margin.to_string()))
    }
    
    fn no_justify(attributes: &mut IndexMap<String, AttributeText>, no_justify: bool) {
        Self::add_attribute(attributes, "nojustify", AttributeText::attr(no_justify.to_string()))
    }

    fn ordering(attributes: &mut IndexMap<String, AttributeText>, ordering: Ordering) {
        Self::add_attribute(attributes, "ordering", AttributeText::attr(ordering.as_slice()))
    }

    fn orientation(attributes: &mut IndexMap<String, AttributeText>, orientation: f32) {
        Self::add_attribute(attributes, "orientation", AttributeText::attr(orientation.to_string()))
    }

    fn pen_width(attributes: &mut IndexMap<String, AttributeText>, pen_width: f32) {
        Self::add_attribute(attributes, "penwidth", AttributeText::attr(pen_width.to_string()))
    }
    
    fn pos(attributes: &mut IndexMap<String, AttributeText>, pos: Point){
        Self::add_attribute(attributes, "pos", AttributeText::attr(pos.to_formatted_string()))
    }

    fn show_boxes(attributes: &mut IndexMap<String, AttributeText>, show_boxes: u32) {
        Self::add_attribute(attributes, "showboxes", AttributeText::attr(show_boxes.to_string()))
    }

    fn sortv(attributes: &mut IndexMap<String, AttributeText>, sortv: u32) {
        Self::add_attribute(attributes, "sortv", AttributeText::attr(sortv.to_string()))
    }

    fn style<'a, S: Into<Cow<'a, str>>>(attributes: &mut IndexMap<String, AttributeText<'a>>, style: S) {
        Self::add_attribute(attributes, "style", AttributeText::attr(style))
    }

    fn target(attributes: &mut IndexMap<String, AttributeText>, target: String) {
        Self::add_attribute(attributes, "target", AttributeText::escaped(target))
    }

    fn tooltip(attributes: &mut IndexMap<String, AttributeText>, tooltip: String) {
        Self::add_attribute(attributes, "tooltip", AttributeText::escaped(tooltip))
    }
    
    fn url(attributes: &mut IndexMap<String, AttributeText>, url: String) {
        Self::add_attribute(attributes, "url", AttributeText::escaped(url))
    }

    fn xlabel(attributes: &mut IndexMap<String, AttributeText>, width: String) {
        Self::add_attribute(attributes, "xlabel", AttributeText::escaped(width))
    }

    fn xlp(attributes: &mut IndexMap<String, AttributeText>, xlp: Point) {
        Self::add_attribute(attributes, "xlp", AttributeText::escaped(xlp.to_formatted_string()))
    }

    fn add_attribute<'a, S: Into<String>>(
        attributes: &mut IndexMap<String, AttributeText<'a>>,
        key: S, 
        value: AttributeText<'a>
    ) {
        attributes.insert(key.into(), value);
    }
}

// fn test_input<Ty>(g: Graph<Ty>) -> io::Result<String> 
// where Ty: GraphType
fn test_input(g: Graph) -> io::Result<String> 
{
    let mut writer = Vec::new();
    let dot = Dot {
        graph: g
    };

    dot.render(&mut writer).unwrap();
    let mut s = String::new();
    Read::read_to_string(&mut &*writer, &mut s)?;
    Ok(s)
}


// TODO: color test

#[test]
fn empty_digraph() {
    // TODO: support both String and &str
    let g = GraphBuilder::new_directed(Some("empty_graph".to_string())).build();
    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"digraph empty_graph {
}
"#
    );
}

#[test]
fn empty_graph() {
    // TODO: support both String and &str
    let g = GraphBuilder::new_undirected(Some("empty_graph".to_string())).build();
    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"graph empty_graph {
}
"#
    );
}

#[test]
fn single_node() {
    let g = GraphBuilder::new_directed(Some("single_node".to_string()))
        .add_node(Node::new("N0".to_string()))
        .build();
    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"digraph single_node {
    N0;
}
"#
    );
}

#[test]
fn single_node_with_style() {
    let node = NodeBuilder::new("N0".to_string())
        .add_attribute("style", AttributeText::quotted("dashed"))
        .build();

    let g = GraphBuilder::new_directed(Some("single_node".to_string()))
        .add_node(node)
        .build();

    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"digraph single_node {
    N0 [style="dashed"];
}
"#
    );
}

#[test]
fn support_non_inline_builder() {
    let mut g = GraphBuilder::new_directed(Some("single_node".to_string()));

    // TODO: having to split this is stupid. am i doing something wrong?
    let mut node_builder = NodeBuilder::new("N0".to_string());
    node_builder.add_attribute("style", AttributeText::quotted("dashed"));

    if true {
        node_builder.add_attribute("foo", AttributeText::quotted("baz"));
    }

    let node = node_builder.build();
    g.add_node(node);

    let r = test_input(g.build());
    assert_eq!(
        r.unwrap(),
        r#"digraph single_node {
    N0 [style="dashed", foo="baz"];
}
"#
    );
}

#[test]
fn builder_support_shape() {
    let node = NodeBuilder::new("N0".to_string())
        .shape(Shape::Note)
        .build();

    let g = GraphBuilder::new_directed(Some("node_shape".to_string()))
        .add_node(node)
        .build();

    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"digraph node_shape {
    N0 [shape=note];
}
"#
    );
}


#[test]
fn single_edge() {
    let g = GraphBuilder::new_directed(Some("single_edge".to_string()))
        .add_node(Node::new("N0".to_string()))
        .add_node(Node::new("N1".to_string()))
        .add_edge(Edge::new("N0".to_string(), "N1".to_string()))
        .build();

    let r = test_input(g);
    
    assert_eq!(
        r.unwrap(),
        r#"digraph single_edge {
    N0;
    N1;
    N0 -> N1;
}
"#
    );
}

#[test]
fn single_edge_with_style() {
    let edge = EdgeBuilder::new("N0".to_string(), "N1".to_string())
        .add_attribute("style", AttributeText::quotted("bold"))
        .build();

    let g = GraphBuilder::new_directed(Some("single_edge".to_string()))
        .add_node(Node::new("N0".to_string()))
        .add_node(Node::new("N1".to_string()))
        .add_edge(edge)
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph single_edge {
    N0;
    N1;
    N0 -> N1 [style="bold"];
}
"#
    );
}

#[test]
fn graph_attributes() {

    let graph_attributes = GraphAttributeStatementBuilder::new().rank_dir(RankDir::LeftRight).build();
    let node_attributes = NodeAttributeStatementBuilder::new().style("filled").build();
    let edge_attributes = EdgeAttributeStatementBuilder::new().min_len(1).build();

    let g = GraphBuilder::new_directed(Some("graph_attributes".to_string()))
        .add_graph_attributes(graph_attributes)
        .add_node_attributes(node_attributes)
        .add_edge_attributes(edge_attributes)
        .add_attribute(AttributeType::None, "ranksep".to_string(), AttributeText::attr("0.5"))
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph graph_attributes {
    graph [rankdir=LR];
    node [style=filled];
    edge [minlen=1];
    ranksep=0.5;
}
"#
    );
}