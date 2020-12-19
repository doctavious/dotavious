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


/// Most of this comes from core rust. Where to provide attribution?
/// The text for a graphviz label on a node or edge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AttributeText<'a> {

    /// Preserves the text directly as is but wrapped in quotes.
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

/// A graph's edge type determines whether it has directed edges or not.
pub trait GraphType {
    fn is_directed() -> bool;

    // TODO: maybe this doesnt below here
    // dont love the name
    fn as_slice() -> &'static str;

    fn edge_slice() -> &'static str;
}

// impl GraphType for Directed {
//     fn is_directed() -> bool {
//         true
//     }

//     fn as_slice() -> &'static str {
//         "digraph"
//     }

//     fn edge_slice() -> &'static str {
//         "->"
//     }
// }

// impl GraphType for Undirected {
//     fn is_directed() -> bool {
//         false
//     }

//     fn as_slice() -> &'static str {
//         "graph"
//     }

//     fn edge_slice() -> &'static str {
//         "--"
//     }
// }

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
        for comment in &self.graph.comments {
            // TODO: split comment into lines of 80 or so characters
            writeln!(w, "// {}", comment)?;
        }

        let strict = if self.graph.strict { "strict " } else { "" }; 
        let id = self.graph.id.as_deref().unwrap_or_default();
        //let edge_op = self.graph.edge_type();
        let edge_op = self.graph.edgeop();

        //writeln!(w, "{}{} {} {{", strict, self.graph.as_slice(), id)?;
        writeln!(w, "{}{} {} {{", strict, self.graph.graph_type(), id)?;


        // TODO: clean this up
        for (key, value) in self.graph.attributes {
            write!(w, "{}", INDENT);
            match key {
                AttributeType::Edge => {
                    write!(w, "edge");
                    if !value.is_empty() {
                        write!(w, " [");
        
                        let mut iter = value.iter();
                        let first = iter.next().unwrap();
                        write!(w, "{}={}", first.0, first.1.to_dot_string());
                        for (key, value) in iter {
                            write!(w, ", ");
                            write!(w, "{}={}", key, value.to_dot_string());
                        }
                        write!(w, "]");
                    }
                    writeln!(w, ";");
                },
                AttributeType::Graph => {
                    write!(w, "graph");
                    if !value.is_empty() {
                        write!(w, " [");
        
                        let mut iter = value.iter();
                        let first = iter.next().unwrap();
                        write!(w, "{}={}", first.0, first.1.to_dot_string());
                        for (key, value) in iter {
                            write!(w, ", ");
                            write!(w, "{}={}", key, value.to_dot_string());
                        }
                        write!(w, "]");
                    }
                    writeln!(w, ";");
                },
                AttributeType::Node => {
                    write!(w, "node");
                    if !value.is_empty() {
                        write!(w, " [");
        
                        let mut iter = value.iter();
                        let first = iter.next().unwrap();
                        write!(w, "{}={}", first.0, first.1.to_dot_string());
                        for (key, value) in iter {
                            write!(w, ", ");
                            write!(w, "{}={}", key, value.to_dot_string());
                        }
                        write!(w, "]");
                    }
                    writeln!(w, ";");
                },
                AttributeType::None => {
                    if !value.is_empty() {        
                        for (key, value) in value.iter() {
                            writeln!(w, "{}={};", key, value.to_dot_string());
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
            writeln!(w, "{}", n.to_dot_string());
        }

        for e in self.graph.edges {
            write!(w, "{}", INDENT);
            write!(w, "{} {} {}", e.source, edge_op, e.target);
            // TODO: render ops
            if !e.attributes.is_empty() {
                write!(w, " [");

                let mut iter = e.attributes.iter();
                let first = iter.next().unwrap();
                write!(w, "{}={}", first.0, first.1.to_dot_string());
                for (key, value) in iter {
                    write!(w, ", ");
                    write!(w, "{}={}", key, value.to_dot_string());
                }
                write!(w, "]");
            }
            writeln!(w, ";");
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

// /// Marker type for a directed graph.
// #[derive(Clone, Copy, Debug)]
// pub enum Directed {}

// /// Marker type for an undirected graph.
// #[derive(Clone, Copy, Debug)]
// pub enum Undirected {}


// pub type DiGraph<'a> = Graph<'a, Directed>;

// pub type UnGraph<'a> = Graph<'a, Undirected>;

pub struct Graph<'a> {
    pub id: Option<String>,
    
    pub is_directed: bool,

    pub strict: bool,

    // Comment added to the first line of the source.
    pub comments: Vec<String>,

    pub attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,

    pub nodes: Vec<Node<'a>>,

    pub edges: Vec<Edge<'a>>,
}

impl<'a> Graph<'a> {

    pub fn new(
        id: Option<String>,
        is_directed: bool,
        strict: bool,
        comments: Vec<String>,
        attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,
        nodes: Vec<Node<'a>>,
        edges: Vec<Edge<'a>>,
    ) -> Self {
        Graph {
            id,
            is_directed,
            strict,
            comments,
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

    pub fn edgeop(&self) -> &'static str {
        if self.is_directed {
            "->"
        } else {
            "--"
        }
    }

}


// pub struct Graph<'a, Ty = Directed> {

//     pub id: Option<String>,
    
//     pub strict: bool,

//     // Comment added to the first line of the source.
//     // TODO: support multiple comments
//     pub comment: Option<String>,

//     pub attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,

//     pub nodes: Vec<Node<'a>>,

//     pub edges: Vec<Edge<'a>>,

//     ty: PhantomData<Ty>,

//     // TODO: should this have
//     // pub graph_type: Ty,
//     // then have Directed and Undirected enums implement fn to print graph type string?
//     // pub graph_type: Ty,
// }

// // TODO: i feel like default should be undirect. 
// // imo, feel more natural to say new_directed vs new_undirected. check to see if 
// impl<'a> Graph<'a, Directed> {
//     pub fn new(id: Option<String>) -> Self {
//         Graph {
//             id: id,
//             strict: false,
//             comment: None,
//             attributes: IndexMap::new(),
//             nodes: Vec::new(),
//             edges: Vec::new(),
//             ty: PhantomData,
//         }
//     }
// }

// impl<'a> Graph<'a, Undirected> {
//     /// Create a new `Graph` with undirected edges.
//     ///
//     /// This is a convenience method. Use `Graph::with_capacity` or `Graph::default` for
//     /// a constructor that is generic in all the type parameters of `Graph`.
//     pub fn new_undirected(id: Option<String>) -> Self {
//         Graph {
//             id: id,
//             strict: false,
//             comment: None,
//             attributes: IndexMap::new(),
//             nodes: Vec::new(),
//             edges: Vec::new(),
//             ty: PhantomData,
//         }
//     }
// }

// impl<'a, Ty> Graph<'a, Ty> 
// where Ty: GraphType
// {
//     /// Whether the graph has directed edges or not.
//     #[inline]
//     pub fn is_directed(&self) -> bool {
//         Ty::is_directed()
//     }

//     pub fn as_slice(&self) -> &'static str {
//         Ty::as_slice()
//     }

//     // graphviz calls this edgeop
//     pub fn edge_type(&self) -> &'static str {
//         Ty::edge_slice()
//     }

//     pub fn add_node(&mut self, node: Node<'a>) -> &mut Self {
//         self.nodes.push(node);
//         self
//     }

//     pub fn add_edge(&mut self, edge: Edge<'a>) -> &mut Self {
//         self.edges.push(edge);
//         self
//     }

//     pub fn attribute(&mut self, attribute_type: AttributeType, key: String, value: AttributeText<'a>) -> &mut Self {
//         println!("attribute type: {:?}", attribute_type);
//         self.attributes.entry(attribute_type).or_insert(IndexMap::new())
//             .insert(key, value);
//         self    
//     }
// }

pub struct GraphBuilder<'a> {
    id: Option<String>,
    
    is_directed: bool,

    strict: bool,

    attributes: IndexMap<AttributeType, IndexMap<String, AttributeText<'a>>>,

    nodes: Vec<Node<'a>>,
    
    edges: Vec<Edge<'a>>,

    // Graphviz says it can only be comment
    comments: Vec<String>,
}

// TODO: id should be an escString
impl<'a> GraphBuilder<'a> {
    pub fn new_directed(id: Option<String>) -> Self {
        Self {
            id: id,
            is_directed: true,
            strict: false,
            attributes: IndexMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            comments: Vec::new(),
        }
    }

    pub fn new_undirected(id: Option<String>) -> Self {
        Self {
            id: id,
            is_directed: false,
            strict: false,
            attributes: IndexMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            comments: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node<'a>) -> &mut Self {
        self.nodes.push(node);
        self
    }

    pub fn add_edge(&mut self, edge: Edge<'a>) -> &mut Self {
        self.edges.push(edge);
        self
    }

    pub fn add_attribute(&mut self, attribute_type: AttributeType, key: String, value: AttributeText<'a>) -> &mut Self {
        self.attributes.entry(attribute_type).or_insert(IndexMap::new())
            .insert(key, value);
        self
    }

    pub fn add_attributes(&mut self, attribute_type: AttributeType, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
        self.attributes.entry(attribute_type).or_insert(IndexMap::new()).extend(attributes);
        self
    }

    pub fn strict(&mut self) -> &mut Self {
        self.strict = true;
        self
    }


    pub fn background(&mut self, background: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("_background"), AttributeText::attr(background))
    }

    /// color
    /// "#%2x%2x%2x"	Red-Green-Blue (RGB)
    /// "#%2x%2x%2x%2x"	Red-Green-Blue-Alpha (RGBA)
    /// "H[, ]+S[, ]+V"	Hue-Saturation-Value (HSV) 0.0 <= H,S,V <= 1.0
    /// string	color name
    /// color list
    /// A colon-separated list of weighted color values: WC(:WC)* where each WC has the form C(;F)? 
    /// with C a color value and the optional F a floating-point number, 0 ≤ F ≤ 1. 
    /// The sum of the floating-point numbers in a colorList must sum to at most 1.
    pub fn background_color(&mut self, background_color: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("bgcolor"), AttributeText::quotted(background_color))
    }

    /// Type: rect which is "%f,%f,%f,%f"
    /// The rectangle llx,lly,urx,ury gives the coordinates, in points, of the lower-left corner (llx,lly) 
    /// and the upper-right corner (urx,ury).
    pub fn bounding_box(&mut self, bounding_box: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("bb"), AttributeText::quotted(bounding_box))
    }

    pub fn center(&mut self, center: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("center"), AttributeText::attr(center.to_string()))
    }

    pub fn charset(&mut self, charset: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("charset"), AttributeText::quotted(charset))
    }

    /// Classnames to attach to the node, edge, graph, or cluster’s SVG element. 
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    pub fn class(&mut self, class: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("class"), AttributeText::quotted(class))
    }

    pub fn cluster_rank(&mut self, cluster_rank: ClusterMode) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("clusterrank"), AttributeText::quotted(cluster_rank.as_slice()))
    }

    /// This attribute specifies a color scheme namespace: the context for interpreting color names.
    /// In particular, if a color value has form "xxx" or "//xxx", then the color xxx will be evaluated 
    /// according to the current color scheme. If no color scheme is set, the standard X11 naming is used.
    /// For example, if colorscheme=bugn9, then color=7 is interpreted as color="/bugn9/7".
    pub fn color_scheme(&mut self, color_scheme: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("colorscheme"), AttributeText::quotted(color_scheme))
    }

    /// Comments are inserted into output. Device-dependent
    pub fn comment(&mut self, comment: String) -> &mut Self {
        self.comments.push(comment);
        self
    }

    pub fn compound(&mut self, compound: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("compound"), AttributeText::quotted(compound))
    }

    pub fn concentrate(&mut self, concentrate: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("concentrate"), AttributeText::quotted(concentrate))
    }

    /// Specifies the expected number of pixels per inch on a display device.
    /// Also known as resolution
    pub fn dpi(&mut self, dpi: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("dpi"), AttributeText::quotted(dpi.to_string()))
    }

    // color
    // color list
    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    pub fn fill_color(&mut self, fill_color: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("fillcolor"), AttributeText::quotted(fill_color))
    }

    // color
    // color list
    /// Color used for text.
    pub fn font_color(&mut self, font_color: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("fontcolor"), AttributeText::quotted(font_color))
    }

    /// Font used for text. 
    pub fn font_name(&mut self, font_name: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("fontname"), AttributeText::quotted(font_name))
    }

    pub fn font_names(&mut self, font_names: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("fontnames"), AttributeText::quotted(font_names))
    }
    
    pub fn font_path(&mut self, font_path: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("fontpath"), AttributeText::quotted(font_path))
    }

    /// Font size, in points, used for text.
    /// default: 14.0, minimum: 1.0
    pub fn font_size(&mut self, font_size: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("fontsize"), AttributeText::quotted(font_size.to_string()))
    }

    pub fn force_label(&mut self, force_label: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("forcelabel"), AttributeText::attr(force_label.to_string()))
    }

    /// If a gradient fill is being used, this determines the angle of the fill.
    pub fn gradient_angle(&mut self, gradient_angle: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("gradientangle"), AttributeText::attr(gradient_angle.to_string()))
    }

    // TODO: delete and just use url?
    /// Synonym for URL.
    pub fn href(&mut self, href: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("href"), AttributeText::escaped(href))
    }

    pub fn image_path(&mut self, image_path: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("imagepath"), AttributeText::escaped(image_path))
    }

    /// An escString or an HTML label.
    pub fn label(&mut self, label: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("label"), AttributeText::quotted(label))
    }

    pub fn label_scheme(&mut self, label_scheme: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("labelscheme"), AttributeText::attr(label_scheme.to_string()))
    }

    // If labeljust=r, the label is right-justified within bounding rectangle
    // If labeljust=l, left-justified
    // Else the label is centered.
    pub fn label_just(&mut self, label_just: LabelJustification) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("labeljust"), AttributeText::attr(label_just.as_slice()))
    }

    // Vertical placement of labels for nodes, root graphs and clusters.
    // For graphs and clusters, only labelloc=t and labelloc=b are allowed, corresponding to placement at the top and bottom, respectively.
    // By default, root graph labels go on the bottom and cluster labels go on the top.
    // Note that a subgraph inherits attributes from its parent. Thus, if the root graph sets labelloc=b, the subgraph inherits this value.
    // For nodes, this attribute is used only when the height of the node is larger than the height of its label.
    // If labelloc=t, labelloc=c, labelloc=b, the label is aligned with the top, centered, or aligned with the bottom of the node, respectively.
    // By default, the label is vertically centered.
    pub fn label_location(&mut self, label_location: LabelLocation) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("labelloc"), AttributeText::attr(label_location.as_slice()))
    }

    pub fn landscape(&mut self, landscape: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("landscape"), AttributeText::attr(landscape.to_string()))
    }

    /// Specifies the separator characters used to split an attribute of type layerRange into a list of ranges.
    pub fn layer_list_sep(&mut self, layer_list_sep: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("layerlistsep"), AttributeText::attr(layer_list_sep))
    }

    /// Specifies a linearly ordered list of layer names attached to the graph
    /// The graph is then output in separate layers. 
    /// Only those components belonging to the current output layer appear.
    pub fn layers(&mut self, layers: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("layers"), AttributeText::attr(layers))
    }

    /// Selects a list of layers to be emitted.
    pub fn layer_select(&mut self, layer_select: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("layerselect"), AttributeText::attr(layer_select))
    }

    /// Specifies the separator characters used to split the layers attribute into a list of layer names.
    /// default: ":\t "
    pub fn layer_sep(&mut self, layer_sep: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("layersep"), AttributeText::attr(layer_sep))
    }

    /// Height of graph or cluster label, in inches.
    pub fn lheight(&mut self, lheight: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("lheight"), AttributeText::attr(lheight.to_string()))
    }

    // TODO: I dont understand this...
    /// "%f,%f('!')?" representing the point (x,y). The optional '!' indicates the node position should not change (input-only).
    /// If dim=3, point may also have the format "%f,%f,%f('!')?" to represent the point (x,y,z).
    pub fn lp(&mut self, lp: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("lp"), AttributeText::attr(lp))
    }

    /// Width of graph or cluster label, in inches.
    pub fn lwidth(&mut self, lwidth: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("lwidth"), AttributeText::attr(lwidth.to_string()))
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
    pub fn margin(&mut self, margin: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("margin"), AttributeText::attr(margin.to_string()))
    }

    /// Multiplicative scale factor used to alter the MinQuit (default = 8) and MaxIter (default = 24) parameters used during crossing minimization.
    /// These correspond to the number of tries without improvement before quitting and the maximum number of iterations in each pass.
    pub fn mclimit(&mut self, mclimit: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("mclimit"), AttributeText::attr(mclimit.to_string()))
    }

    /// Specifies the minimum separation between all nodes.
    pub fn mindist(&mut self, mindist: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("mindist"), AttributeText::attr(mindist.to_string()))
    }

    /// Whether to use a single global ranking, ignoring clusters.
    /// The original ranking algorithm in dot is recursive on clusters. 
    /// This can produce fewer ranks and a more compact layout, but sometimes at the cost of a head node being place on a higher rank than the tail node. 
    /// It also assumes that a node is not constrained in separate, incompatible subgraphs. 
    /// For example, a node cannot be in a cluster and also be constrained by rank=same with a node not in the cluster.
    /// This allows nodes to be subject to multiple constraints. 
    /// Rank constraints will usually take precedence over edge constraints.
    pub fn newrank(&mut self, newrank: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("newrank"), AttributeText::attr(newrank.to_string()))
    }

    // TODO: add constraint
    /// specifies the minimum space between two adjacent nodes in the same rank, in inches.
    /// default: 0.25, minimum: 0.02
    pub fn nodesep(&mut self, nodesep: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("nodesep"), AttributeText::attr(nodesep.to_string()))
    }

    /// By default, the justification of multi-line labels is done within the largest context that makes sense. 
    /// Thus, in the label of a polygonal node, a left-justified line will align with the left side of the node (shifted by the prescribed margin). 
    /// In record nodes, left-justified line will line up with the left side of the enclosing column of fields. 
    /// If nojustify=true, multi-line labels will be justified in the context of itself.
    /// For example, if nojustify is set, the first label line is long, and the second is shorter and left-justified, 
    /// the second will align with the left-most character in the first line, regardless of how large the node might be.
    pub fn no_justify(&mut self, no_justify: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("nojustify"), AttributeText::attr(no_justify.to_string()))
    }

    /// Sets number of iterations in network simplex applications.
    /// nslimit is used in computing node x coordinates.
    /// If defined, # iterations = nslimit * # nodes; otherwise, # iterations = MAXINT.
    pub fn nslimit(&mut self, nslimit: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("nslimit"), AttributeText::attr(nslimit.to_string()))
    }

    /// If ordering="out", then the outedges of a node, that is, edges with the node as its tail node, must appear left-to-right in the same order in which they are defined in the input.
    /// If ordering="in", then the inedges of a node must appear left-to-right in the same order in which they are defined in the input.
    /// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph or subgraph.
    /// Note that the graph attribute takes precedence over the node attribute.
    pub fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("ordering"), AttributeText::attr(ordering.as_slice()))
    }

    // TODO: constrain to 0 - 360. Docs say min is 360 which should be max right?
    /// When used on nodes: Angle, in degrees, to rotate polygon node shapes. 
    /// For any number of polygon sides, 0 degrees rotation results in a flat base.
    /// When used on graphs: If "[lL]*", sets graph orientation to landscape.
    /// Used only if rotate is not defined.
    /// Default: 0.0 and minimum: 360.0
    pub fn orientation(&mut self, orientation: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("orientation"), AttributeText::attr(orientation.to_string()))
    }

    /// Specify order in which nodes and edges are drawn.
    /// default: breadthfirst
    pub fn output_order(&mut self, output_order: OutputMode) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("outputorder"), AttributeText::attr(output_order.as_slice()))
    }

    /// Whether each connected component of the graph should be laid out separately, and then the graphs packed together.
    /// If false, the entire graph is laid out together. 
    /// The granularity and method of packing is influenced by the packmode attribute.
    pub fn pack(&mut self, pack: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("pack"), AttributeText::attr(pack.to_string()))
    }

    // TODO: constrain to non-negative integer.
    /// Whether each connected component of the graph should be laid out separately, and then the graphs packed together.
    /// This is used as the size, in points,of a margin around each part; otherwise, a default margin of 8 is used.
    /// pack is treated as true if the value of pack iso a non-negative integer.
    pub fn pack_int(&mut self, pack: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("pack"), AttributeText::attr(pack.to_string()))
    }

    /// This indicates how connected components should be packed (cf. packMode). 
    /// Note that defining packmode will automatically turn on packing as though one had set pack=true.
    pub fn pack_mode(&mut self, pack_mode: PackMode) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("packmode"), AttributeText::attr(pack_mode.as_slice()))
    }

    /// Specifies how much, in inches, to extend the drawing area around the minimal area needed to draw the graph.
    /// Both the x and y pad values are set equal to the given value. 
    /// This area is part of the drawing and will be filled with the background color, if appropriate.
    /// default: 0.0555
    pub fn pad(&mut self, pad: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("pad"), AttributeText::attr(pad.to_string()))
    }

    /// Specifies how much, in inches, to extend the drawing area around the minimal area needed to draw the graph.
    pub fn pad_point(&mut self, pad: Point) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("pad"), AttributeText::attr(pad.to_formatted_string()))
    }

    /// Width and height of output pages, in inches.
    /// Value given is used for both the width and height.
    pub fn page(&mut self, page: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("page"), AttributeText::attr(page.to_string()))
    }

    /// Width and height of output pages, in inches.
    pub fn page_point(&mut self, page: Point) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("page"), AttributeText::attr(page.to_formatted_string()))
    }

    /// The order in which pages are emitted.
    /// Used only if page is set and applicable.
    /// Limited to one of the 8 row or column major orders.
    pub fn page_dir(&mut self, page_dir: PageDirection) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("pagedir"), AttributeText::attr(page_dir.as_slice()))
    }

    // TODO: constrain
    /// If quantum > 0.0, node label dimensions will be rounded to integral multiples of the quantum.
    /// default: 0.0, minimum: 0.0
    pub fn quantum(&mut self, quantum: f32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("quantum"), AttributeText::attr(quantum.to_string()))
    }

    /// Sets direction of graph layout.
    /// For example, if rankdir="LR", and barring cycles, an edge T -> H; will go from left to right. 
    /// By default, graphs are laid out from top to bottom.
    /// This attribute also has a side-effect in determining how record nodes are interpreted. 
    /// See record shapes.
    pub fn rank_dir(&mut self, rank_dir: RankDir) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("rankdir"), AttributeText::attr(rank_dir.as_slice()))
    }

    /// sets the desired rank separation, in inches.
    /// This is the minimum vertical distance between the bottom of the nodes in one rank 
    /// and the tops of nodes in the next. If the value contains equally, 
    /// the centers of all ranks are spaced equally apart. 
    /// Note that both settings are possible, e.g., ranksep="1.2 equally".
    pub fn rank_sep(&mut self, rank_sep: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("ranksep"), AttributeText::attr(rank_sep))
    }

    // TODO: numeric vs string
    // Strings: fill, compress, expand, auto
    /// Sets the aspect ratio (drawing height/drawing width) for the drawing.
    /// Note that this is adjusted before the size attribute constraints are enforced.
    pub fn ratio(&mut self, ratio: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("ratio"), AttributeText::attr(ratio))
    }

    /// If true and there are multiple clusters, run crossing minimization a second time.
    pub fn remincross(&mut self, remincross: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("remincross"), AttributeText::attr(remincross.to_string()))
    }

    /// If rotate=90, sets drawing orientation to landscape.
    pub fn rotate(&mut self, rotate: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("rotate"), AttributeText::attr(rotate.to_string()))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at the end if showboxes=2.
    /// (Debugging, TB mode only!)
    /// default: 0, minimum: 0
    pub fn show_boxes(&mut self, show_boxes: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("showboxes"), AttributeText::attr(show_boxes.to_string()))
    }

    /// Maximum width and height of drawing, in inches.
    /// Value used for both the width and the height.
    /// If defined and the drawing is larger than the given size, the drawing 
    /// is uniformly scaled down so that it fits within the given size.
    /// If desired_min is true, and both both dimensions of the drawing 
    /// are less than size, the drawing is scaled up uniformly until at 
    /// least one dimension equals its dimension in size.
    pub fn size(&mut self, size: u32, desired_min: bool) -> &mut Self {
        let mut text = format!("{}", size);
        if desired_min {
            text.push_str("!");
        }
        self.add_attribute(AttributeType::Graph, String::from("size"), AttributeText::attr(text))
    }

    /// Maximum width and height of drawing, in inches.
    /// If defined and the drawing is larger than the given size, the drawing 
    /// is uniformly scaled down so that it fits within the given size.
    /// If desired_min is true, and both both dimensions of the drawing 
    /// are less than size, the drawing is scaled up uniformly until at 
    /// least one dimension equals its dimension in size.
    pub fn size_point(&mut self, size: Point, desired_min: bool) -> &mut Self {
        let mut text = size.to_formatted_string();
        if desired_min {
            text.push_str("!");
        }
        self.add_attribute(AttributeType::Graph, String::from("size"), AttributeText::attr(text))
    }

    /// If packmode indicates an array packing, sortv specifies an insertion order 
    /// among the components, with smaller values inserted first.
    /// default: 0, minimum: 0
    pub fn sortv(&mut self, sortv: u32) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("sortv"), AttributeText::attr(sortv.to_string()))
    }

    /// Controls how, and if, edges are represented.
    /// If splines=true, edges are drawn as splines routed around nodes; 
    /// if splines=false, edges are drawn as line segments.
    pub fn splines_bool(&mut self, splines: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("splines"), AttributeText::attr(splines.to_string()))
    }

    /// Controls how, and if, edges are represented.
    pub fn splines(&mut self, splines: Splines) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("splines"), AttributeText::attr(splines.as_slice()))
    }

    /// Set style information for components of the graph.
    pub fn style(&mut self, style: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("style"), AttributeText::attr(style))
    }

    /// A URL or pathname specifying an XML style sheet, used in SVG output.
    /// Combine with class to style elements using CSS selectors.
    pub fn stylesheet(&mut self, stylesheet: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("stylesheet"), AttributeText::attr(stylesheet))
    }

    /// If the object has a URL, this attribute determines which window of the browser is used for the URL.
    pub fn target(&mut self, target: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("target"), AttributeText::escaped(target))
    }

    /// Whether internal bitmap rendering relies on a truecolor color model or uses a color palette.
    /// If truecolor is unset, truecolor is not used unless there is a shapefile property for some node in the graph. The output model will use the input model when possible.
    pub fn true_color(&mut self, true_color: bool) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("truecolor"), AttributeText::attr(true_color.to_string()))
    }

    pub fn url(&mut self, url: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("url"), AttributeText::escaped(url))
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
    pub fn viewport(&mut self, viewport: String) -> &mut Self {
        self.add_attribute(AttributeType::Graph, String::from("viewport"), AttributeText::attr(viewport))
    }

    pub fn build(&self) -> Graph<'a> {
        Graph {
            id: self.id.to_owned(),
            is_directed: self.is_directed,
            strict: self.strict,
            comments: self.comments.clone(), // TODO: is clone the only option here?
            attributes: self.attributes.clone(), // TODO: is clone the only option here?
            nodes: self.nodes.clone(), // TODO: is clone the only option here?
            edges: self.edges.clone(), // TODO: is clone the only option here?
        }
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
    pub can_change: bool,
}

impl Point {

    pub fn new_2d(x: f32, y: f32, can_change: bool) -> Self {
        Self {
            x,
            y,
            z: None,
            can_change,
        }
    }

    pub fn new_3d(x: f32, y: f32, z: f32, can_change: bool) -> Self {
        Self {
            x,
            y,
            z: Some(z),
            can_change,
        }
    }

    pub fn to_formatted_string(self) -> String { 
        let mut slice = format!("{},{}", self.x, self.y);
        if self.z.is_some() {
            slice.push_str(format!(",{}", self.z.unwrap()).as_str());
        }
        if !self.can_change {
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

// TODO: add node builder using "with" convention
#[derive(Clone, Debug)]
pub struct Node<'a> {

    pub id: String,

    pub port: Option<String>,

    // pub compass: Option<Compass>,

    // // TODO: enum?
    // shape: Option<String>,

    // label: Option<String>,
    
    pub attributes: IndexMap<String, AttributeText<'a>>,

    // style

}

impl<'a> Node<'a> {

    pub fn new(id: String) -> Node<'a> {
        // TODO: constrain id
        Node {
            id: id,
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

    pub fn label(mut self, text: String) -> Self {
        // self.label = Some(text);
        self.attributes.insert("label".to_string(), QuottedStr(text.into()));
        self
    }

    /// Add an attribute to the node.
    pub fn attribute(mut self, key: String, value: AttributeText<'a>) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Add multiple attribures to the node.
    pub fn attributes(mut self, attributes: HashMap<String, AttributeText<'a>>) -> Self {
        self.attributes.extend(attributes);
        self
    }

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

    comment: Option<String>,
    
    attributes: IndexMap<String, AttributeText<'a>>,
}

impl<'a> NodeBuilder<'a> {
    pub fn new(id: String) -> Self {
        Self {
            id: id,
            port: None,
            comment: None,
            attributes: IndexMap::new(),
        }
    }

    /// Set the port for the node.
    pub fn port<S: Into<String>>(&mut self, port: S) -> &mut Self {
        self.port = Some(port.into());
        self
    }

    // pub fn compass<'a>(&'a mut self, compass: Compass) -> &'a mut Node {
    //     self.compass = Some(compass);
    //     self
    // }

    // TODO: constrain
    /// Indicates the preferred area for a node or empty cluster when laid out by patchwork.
    /// default: 1.0, minimum: >0
    pub fn area(&mut self, area: f32) -> &mut Self {
        self.attributes.insert(String::from("area"), AttributeText::attr(area.to_string()));
        self
    }

    /// Classnames to attach to the node, edge, graph, or cluster’s SVG element. 
    /// Combine with stylesheet for styling SVG output using CSS classnames.
    /// Multiple space-separated classes are supported.
    pub fn class(&mut self, class: String) -> &mut Self {
        self.add_attribute(String::from("class"), AttributeText::quotted(class))
    }

    // color / color list
    /// Basic drawing color for graphics, not text. For the latter, use the fontcolor attribute.
    pub fn color(&mut self, color: String) -> &mut Self {
        self.add_attribute(String::from("color"), AttributeText::quotted(color))
    }

    /// This attribute specifies a color scheme namespace: the context for interpreting color names.
    /// In particular, if a color value has form "xxx" or "//xxx", then the color xxx will be evaluated 
    /// according to the current color scheme. If no color scheme is set, the standard X11 naming is used.
    /// For example, if colorscheme=bugn9, then color=7 is interpreted as color="/bugn9/7".
    pub fn color_scheme(&mut self, color_scheme: String) -> &mut Self {
        self.add_attribute(String::from("colorscheme"), AttributeText::quotted(color_scheme))
    }

    /// Comments are inserted into output. Device-dependent
    pub fn comment(&mut self, comment: String) -> &mut Self {
        self.comment = Some(comment);
        self
    }

    /// Distortion factor for shape=polygon.
    /// Positive values cause top part to be larger than bottom; negative values do the opposite.
    pub fn distortion(&mut self, distortion: f32) -> &mut Self {
        self.add_attribute(String::from("distortion"), AttributeText::attr(distortion.to_string()))
    }

    // color
    // color list
    /// Color used to fill the background of a node or cluster assuming style=filled, or a filled arrowhead.
    pub fn fill_color(&mut self, fill_color: String) -> &mut Self {
        self.add_attribute(String::from("fillcolor"), AttributeText::quotted(fill_color))
    }

    /// If true, the node size is specified by the values of the width and height attributes only and 
    /// is not expanded to contain the text label. 
    /// There will be a warning if the label (with margin) cannot fit within these limits.
    /// If false, the size of a node is determined by smallest width and height needed to contain its label 
    /// and image, if any, with a margin specified by the margin attribute.
    pub fn fixed_size(&mut self, fixed_size: bool) -> &mut Self {
        self.add_attribute(String::from("fixedsize"), AttributeText::quotted(fixed_size.to_string()))
    }

    // color
    // color list
    /// Color used for text.
    pub fn font_color(&mut self, font_color: String) -> &mut Self {
        self.add_attribute(String::from("fontcolor"), AttributeText::quotted(font_color))
    }

    /// Font used for text. 
    pub fn font_name(&mut self, font_name: String) -> &mut Self {
        self.add_attribute(String::from("fontname"), AttributeText::quotted(font_name))
    }

    /// Font size, in points, used for text.
    /// default: 14.0, minimum: 1.0
    pub fn font_size(&mut self, font_size: f32) -> &mut Self {
        self.add_attribute(String::from("fontsize"), AttributeText::quotted(font_size.to_string()))
    }

    /// If a gradient fill is being used, this determines the angle of the fill.
    pub fn gradient_angle(&mut self, gradient_angle: u32) -> &mut Self {
        self.add_attribute(String::from("gradientangle"), AttributeText::attr(gradient_angle.to_string()))
    }

    /// If the end points of an edge belong to the same group, i.e., have the same group attribute, 
    /// parameters are set to avoid crossings and keep the edges straight.
    pub fn group(&mut self, group: String) -> &mut Self {
        self.add_attribute(String::from("group"), AttributeText::attr(group))
    }

    // TODO: constrain
    /// Height of node, in inches.
    /// default: 0.5, minimum: 0.02
    pub fn height(&mut self, height: f32) -> &mut Self {
        self.add_attribute(String::from("height"), AttributeText::attr(height.to_string()))
    }

    // TODO: delete and just use url?
    /// Synonym for URL.
    pub fn href(&mut self, href: String) -> &mut Self {
        self.add_attribute(String::from("href"), AttributeText::escaped(href))
    }

    /// Gives the name of a file containing an image to be displayed inside a node. 
    /// The image file must be in one of the recognized formats, 
    /// typically JPEG, PNG, GIF, BMP, SVG, or Postscript, and be able to be converted 
    /// into the desired output format.
    pub fn image(&mut self, image: String) -> &mut Self {
        self.add_attribute(String::from("image"), AttributeText::quotted(image))
    }

    /// Controls how an image is positioned within its containing node.
    /// Only has an effect when the image is smaller than the containing node.
    pub fn image_pos(&mut self, image_pos: ImagePosition) -> &mut Self {
        self.add_attribute(String::from("imagepos"), AttributeText::quotted(image_pos.as_slice()))
    }

    /// Controls how an image fills its containing node.
    pub fn image_scale_bool(&mut self, image_scale: bool) -> &mut Self {
        self.add_attribute(String::from("imagescale"), AttributeText::quotted(image_scale.to_string()))
    }

    /// Controls how an image fills its containing node.
    pub fn image_scale(&mut self, image_scale: ImageScale) -> &mut Self {
        self.add_attribute(String::from("imagescale"), AttributeText::quotted(image_scale.as_slice()))
    }

    /// Text label attached to objects.
    pub fn label<S: Into<Cow<'a, str>>>(&mut self, text: S) -> &mut Self {
        self.attributes.insert(String::from("label"), QuottedStr(text.into()));
        self
    }

    // Vertical placement of labels for nodes, root graphs and clusters.
    // For graphs and clusters, only labelloc=t and labelloc=b are allowed, corresponding to placement at the top and bottom, respectively.
    // By default, root graph labels go on the bottom and cluster labels go on the top.
    // Note that a subgraph inherits attributes from its parent. Thus, if the root graph sets labelloc=b, the subgraph inherits this value.
    // For nodes, this attribute is used only when the height of the node is larger than the height of its label.
    // If labelloc=t, labelloc=c, labelloc=b, the label is aligned with the top, centered, or aligned with the bottom of the node, respectively.
    // By default, the label is vertically centered.
    pub fn label_location(&mut self, label_location: LabelLocation) -> &mut Self {
        self.add_attribute(String::from("labelloc"), AttributeText::attr(label_location.as_slice()))
    }

    /// Specifies layers in which the node, edge or cluster is present.
    pub fn layer(&mut self, layer: String) -> &mut Self {
        self.add_attribute(String::from("layer"), AttributeText::attr(layer))
    }

    // TODO: point
    /// For nodes, this attribute specifies space left around the node’s label.
    /// If the margin is a single double, both margins are set equal to the given value.
    /// Note that the margin is not part of the drawing but just empty space left around the drawing. 
    /// The margin basically corresponds to a translation of drawing, as would be necessary to center a drawing on a page. 
    /// Nothing is actually drawn in the margin. To actually extend the background of a drawing, see the pad attribute.
    /// By default, the value is 0.11,0.055.
    pub fn margin(&mut self, margin: f32) -> &mut Self {
        self.add_attribute(String::from("margin"), AttributeText::attr(margin.to_string()))
    }

    /// By default, the justification of multi-line labels is done within the largest context that makes sense. 
    /// Thus, in the label of a polygonal node, a left-justified line will align with the left side of the node (shifted by the prescribed margin). 
    /// In record nodes, left-justified line will line up with the left side of the enclosing column of fields. 
    /// If nojustify=true, multi-line labels will be justified in the context of itself.
    /// For example, if nojustify is set, the first label line is long, and the second is shorter and left-justified, 
    /// the second will align with the left-most character in the first line, regardless of how large the node might be.
    pub fn no_justify(&mut self, no_justify: bool) -> &mut Self {
        self.add_attribute(String::from("nojustify"), AttributeText::attr(no_justify.to_string()))
    }

    /// If ordering="out", then the outedges of a node, that is, edges with the node as its tail node, must appear left-to-right in the same order in which they are defined in the input.
    /// If ordering="in", then the inedges of a node must appear left-to-right in the same order in which they are defined in the input.
    /// If defined as a graph or subgraph attribute, the value is applied to all nodes in the graph or subgraph.
    /// Note that the graph attribute takes precedence over the node attribute.
    pub fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        self.add_attribute(String::from("ordering"), AttributeText::attr(ordering.as_slice()))
    }

    // TODO: constrain to 0 - 360. Docs say min is 360 which should be max right?
    /// When used on nodes: Angle, in degrees, to rotate polygon node shapes. 
    /// For any number of polygon sides, 0 degrees rotation results in a flat base.
    /// When used on graphs: If "[lL]*", sets graph orientation to landscape.
    /// Used only if rotate is not defined.
    /// Default: 0.0 and minimum: 360.0
    pub fn orientation(&mut self, orientation: f32) -> &mut Self {
        self.add_attribute(String::from("orientation"), AttributeText::attr(orientation.to_string()))
    }

    /// Specifies the width of the pen, in points, used to draw lines and curves, 
    /// including the boundaries of edges and clusters.
    /// default: 1.0, minimum: 0.0
    pub fn pen_width(&mut self, pen_width: f32) -> &mut Self {
        self.add_attribute(String::from("penwidth"), AttributeText::attr(pen_width.to_string()))
    }

    /// Set number of peripheries used in polygonal shapes and cluster boundaries.
    pub fn peripheries(&mut self, pen_width: u32) -> &mut Self {
        self.add_attribute(String::from("penwidth"), AttributeText::attr(pen_width.to_string()))
    }

    /// Position of node, or spline control points.
    /// the position indicates the center of the node. On output, the coordinates are in points.
    pub fn pos(&mut self, pos: Point) -> &mut Self {
        self.add_attribute(String::from("pos"), AttributeText::attr(pos.to_formatted_string()))
    }

    // TODO: add post_spline

    // TODO: add rect type?
    // "%f,%f,%f,%f"
    /// Rectangles for fields of records, in points.
    pub fn rects(&mut self, rects: String) -> &mut Self {
        self.add_attribute(String::from("rects"), AttributeText::attr(rects))
    }

    /// If true, force polygon to be regular, i.e., the vertices of the polygon will 
    /// lie on a circle whose center is the center of the node.
    pub fn regular(&mut self, regular: bool) -> &mut Self {
        self.add_attribute(String::from("regular"), AttributeText::attr(regular.to_string()))
    }

    /// Gives the number of points used for a circle/ellipse node.
    pub fn sample_points(&mut self, sample_points: u32) -> &mut Self {
        self.add_attribute(String::from("samplepoints"), AttributeText::attr(sample_points.to_string()))
    }

    /// Sets the shape of a node.
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.add_attribute(String::from("shape"), AttributeText::attr(shape.as_slice()))
    }

    // TODO: constrain
    /// Print guide boxes in PostScript at the beginning of routesplines if showboxes=1, or at the end if showboxes=2.
    /// (Debugging, TB mode only!)
    /// default: 0, minimum: 0
    pub fn show_boxes(&mut self, show_boxes: u32) -> &mut Self {
        self.add_attribute(String::from("showboxes"), AttributeText::attr(show_boxes.to_string()))
    }

    /// Number of sides when shape=polygon.
    pub fn sides(&mut self, sides: u32) -> &mut Self {
        self.add_attribute(String::from("sides"), AttributeText::attr(sides.to_string()))
    }

    // TODO: constrain
    /// Skew factor for shape=polygon.
    /// Positive values skew top of polygon to right; negative to left.
    /// default: 0.0, minimum: -100.0
    pub fn skew(&mut self, skew: f32) -> &mut Self {
        self.add_attribute(String::from("skew"), AttributeText::attr(skew.to_string()))
    }

    /// If packmode indicates an array packing, sortv specifies an insertion order 
    /// among the components, with smaller values inserted first.
    /// default: 0, minimum: 0
    pub fn sortv(&mut self, sortv: u32) -> &mut Self {
        self.add_attribute(String::from("sortv"), AttributeText::attr(sortv.to_string()))
    }

    /// Set style information for components of the graph.
    pub fn style(&mut self, style: String) -> &mut Self {
        self.add_attribute(String::from("style"), AttributeText::attr(style))
    }

    /// Add an attribute to the node.
    pub fn add_attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the node.
    pub fn add_attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
        self.attributes.extend(attributes);
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

    pub attributes: HashMap<String, AttributeText<'a>>,
}

impl<'a> Edge<'a> {

    pub fn new(source: String, target: String) -> Edge<'a> {
        Edge {
            source,
            target,
            attributes: HashMap::new(),
        }
    }
}

pub struct EdgeBuilder<'a> {
    pub source: String,

    pub target: String,
    
    attributes: HashMap<String, AttributeText<'a>>,
}

impl<'a> EdgeBuilder<'a> {
    pub fn new(source: String, target: String) -> EdgeBuilder<'a> {
        EdgeBuilder {
            source,
            target,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the edge.
    pub fn attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the edge.
    pub fn attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
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


#[test]
fn empty_digraph() {
    // TODO: support both String and &str
    // let g = Graph::new(Some("empty_graph".to_string()));
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
        .attribute("style", AttributeText::quotted("bold"))
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
    let g = GraphBuilder::new_directed(Some("graph_attributes".to_string()))
        .add_attribute(AttributeType::None, "ranksep".to_string(), AttributeText::attr("0.5"))
        .add_attribute(AttributeType::Graph, "rankdir".to_string(), AttributeText::attr("LR"))
        .add_attribute(AttributeType::Edge, "minlen".to_string(), AttributeText::attr("1"))
        .add_attribute(AttributeType::Node, "style".to_string(), AttributeText::attr("filled"))
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph graph_attributes {
    ranksep=0.5;
    graph [rankdir=LR];
    edge [minlen=1];
    node [style=filled];
}
"#
    );
}