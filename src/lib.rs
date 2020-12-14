//! Simple graphviz dot file format output.

use AttributeText::*;
use std;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::marker::PhantomData;

static INDENT: &str = "    ";

// TODO: should we use a hashmap that retains insertion order?

// TODO: support adding edge based on index of nodes?


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

impl GraphType for Directed {
    fn is_directed() -> bool {
        true
    }

    fn as_slice() -> &'static str {
        "digraph"
    }

    fn edge_slice() -> &'static str {
        "->"
    }
}

impl GraphType for Undirected {
    fn is_directed() -> bool {
        false
    }

    fn as_slice() -> &'static str {
        "graph"
    }

    fn edge_slice() -> &'static str {
        "--"
    }
}

// TODO: probably dont need this struct and can move impl methods into lib module
pub struct Dot<'a, Ty> {
    graph: Graph<'a, Ty>,
    //config: Config,
}

impl<'a, Ty> Dot<'a, Ty> 
where Ty: GraphType
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
        if self.graph.comment.is_some() {
            writeln!(w, "{}", self.graph.comment.as_deref().unwrap_or_default())?;
        }
        
        let strict = if self.graph.strict { "strict " } else { "" }; 
        let id = self.graph.id.as_deref().unwrap_or_default();
        let edge_op = self.graph.edge_type();

        writeln!(w, "{}{} {} {{", strict, self.graph.as_slice(), id)?;

        // TODO: add global graph attributes
        for a in self.graph.attributes {

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

enum AttributeType {
    Graph,
    Node,
    Edge,
    None
}

/// Marker type for a directed graph.
#[derive(Clone, Copy, Debug)]
pub enum Directed {}

/// Marker type for an undirected graph.
#[derive(Clone, Copy, Debug)]
pub enum Undirected {}


pub type DiGraph<'a> = Graph<'a, Directed>;

pub type UnGraph<'a> = Graph<'a, Undirected>;

pub struct Graph<'a, Ty = Directed> {

    pub id: Option<String>,
    
    pub strict: bool,

    // Comment added to the first line of the source.
    // TODO: support multiple comments
    pub comment: Option<String>,

    pub attributes: HashMap<String, AttributeText<'a>>,

    pub nodes: Vec<Node<'a>>,

    pub edges: Vec<Edge<'a>>,

    ty: PhantomData<Ty>,

    // TODO: should this have
    // pub graph_type: Ty,
    // then have Directed and Undirected enums implement fn to print graph type string?
    // pub graph_type: Ty,
}

// TODO: i feel like default should be undirect. 
// imo, feel more natural to say new_directed vs new_undirected. check to see if 
impl<'a> Graph<'a, Directed> {
    pub fn new(id: Option<String>) -> Self {
        Graph {
            id: id,
            strict: false,
            comment: None,
            attributes: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            ty: PhantomData,
        }
    }
}

impl<'a> Graph<'a, Undirected> {
    /// Create a new `Graph` with undirected edges.
    ///
    /// This is a convenience method. Use `Graph::with_capacity` or `Graph::default` for
    /// a constructor that is generic in all the type parameters of `Graph`.
    pub fn new_undirected(id: Option<String>) -> Self {
        Graph {
            id: id,
            strict: false,
            comment: None,
            attributes: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            ty: PhantomData,
        }
    }
}

impl<'a, Ty> Graph<'a, Ty> 
where Ty: GraphType
{
    /// Whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    pub fn as_slice(&self) -> &'static str {
        Ty::as_slice()
    }

    // graphviz calls this edgeop
    pub fn edge_type(&self) -> &'static str {
        Ty::edge_slice()
    }

    pub fn add_node(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge<'a>) {
        self.edges.push(edge);
    }
}

// TODO: add node builder using "with" convention
pub struct Node<'a> {

    pub id: String,

    pub port: Option<String>,

    // pub compass: Option<Compass>,

    // // TODO: enum?
    // shape: Option<String>,

    // label: Option<String>,
    
    pub attributes: HashMap<String, AttributeText<'a>>,

    // style

}

impl<'a> Node<'a> {

    pub fn new(id: String) -> Node<'a> {
        // TODO: constrain id
        Node {
            id: id,
            port: None,
            attributes: HashMap::new(),
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
    
    attributes: HashMap<String, AttributeText<'a>>,
}

impl<'a> NodeBuilder<'a> {
    pub fn new(id: String) -> Self {
        Self {
            id: id,
            port: None,
            attributes: HashMap::new(),
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

    pub fn label<S: Into<Cow<'a, str>>>(&mut self, text: S) -> &mut Self {
        // self.label = Some(text);
        self.attributes.insert("label".to_string(), QuottedStr(text.into()));
        self
    }

    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.attributes.insert("shape".to_string(), AttrStr(shape.as_slice().into()));
        self
    }

    /// Add an attribute to the node.
    pub fn attribute<S: Into<String>>(&mut self, key: S, value: AttributeText<'a>) -> &mut Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add multiple attribures to the node.
    pub fn attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &mut Self {
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
            Shape::Box3D => "box3D",
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

fn test_input<Ty>(g: Graph<Ty>) -> io::Result<String> 
where Ty: GraphType
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
    let g = Graph::new(Some("empty_graph".to_string()));
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
    let g = Graph::new_undirected(Some("empty_graph".to_string()));
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
    let mut g = Graph::new(Some("single_node".to_string()));
    g.add_node(Node::new("N0".to_string()));
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
    let mut g = Graph::new(Some("single_node".to_string()));
    let node = NodeBuilder::new("N0".to_string())
        .attribute("style", AttributeText::quotted("dashed"))
        .build();

    g.add_node(node);

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
    let mut g = Graph::new(Some("single_node".to_string()));

    // TODO: having to split this is stupid. am i doing something wrong?
    let mut node_builder = NodeBuilder::new("N0".to_string());
    node_builder.attribute("style", AttributeText::quotted("dashed"));

    if true {
        node_builder.attribute("foo", AttributeText::quotted("baz"));
    }

    let node = node_builder.build();
    g.add_node(node);

    let r = test_input(g);
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
    let mut g = Graph::new(Some("node_shape".to_string()));

    // TODO: having to split this is stupid. am i doing something wrong?
    let mut node_builder = NodeBuilder::new("N0".to_string());
    node_builder.shape(Shape::Note);

    g.add_node(node_builder.build());

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
    let mut g = Graph::new(Some("single_edge".to_string()));
    g.add_node(Node::new("N0".to_string()));
    g.add_node(Node::new("N1".to_string()));

    g.add_edge(Edge::new("N0".to_string(), "N1".to_string()));

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

    let mut g = Graph::new(Some("single_edge".to_string()));
    g.add_node(Node::new("N0".to_string()));
    g.add_node(Node::new("N1".to_string()));

    let edge = EdgeBuilder::new("N0".to_string(), "N1".to_string())
        .attribute("style", AttributeText::quotted("bold"))
        .build();

    g.add_edge(edge);

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