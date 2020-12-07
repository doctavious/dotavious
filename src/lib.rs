//! Simple graphviz dot file format output.

use AttributeText::*;
use std;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::io::{
    Write
};
use std::marker::PhantomData;

static INDENT: &str = "    ";

/// Most of this comes from core rust. Where to provide attribution?
/// The text for a graphviz label on a node or edge.
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

enum GraphType {
    Graph,
    Digraph
}

impl GraphType {
    pub fn as_slice(self) -> &'static str {
        match self {
            GraphType::Graph => "graph",
            GraphType::Digraph => "digraph",
        }
    }

    // TODO: not sure if I like this or not
    pub fn edge_slice(self) -> &'static str {
        match self {
            GraphType::Graph => "--",
            GraphType::Digraph => "->",
        }
    }
}

// TODO: probably dont need this struct and can move impl methods into lib module
pub struct Dot<'a> {
    graph: Graph<'a>,
    config: Config,
}

impl<'a> Dot<'a> {

    /// Renders directed graph `g` into the writer `w` in DOT syntax.
    /// (Simple wrapper around `render_opts` that passes a default set of options.)
    pub fn render<W>(self, g: Graph, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        // TODO: use default_options?
        self.render_opts(g, w, &[])
    }

    // io::Result<()> vs Result<(), Box<dyn Error>>
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-can-be-used-in-functions-that-return-result
    /// Renders directed graph `g` into the writer `w` in DOT syntax.
    /// (Main entry point for the library.)
    pub fn render_opts<W>(self, graph: Graph, w: &mut W, options: &[RenderOption]) -> io::Result<()>
    where
        W: Write,
    {
        writeln!(w, "{}", graph.comment.unwrap_or_default())?;

        let strict = if graph.strict { "strict " } else { "" }; 
        let id = graph.id.unwrap_or_default();
        
        // TODO: implement
        // writeln!(w, "{}{} {}{{", strict, graph.graph_type.as_slice(), id)?;

        // Global graph properties
        if options.contains(&RenderOption::Monospace) {
            writeln!(w, r#"    graph[fontname="monospace"];"#)?;
            writeln!(w, r#"    node[fontname="monospace"];"#)?;
            writeln!(w, r#"    edge[fontname="monospace"];"#)?;
        }

        for n in graph.nodes {
            // TODO: handle render options
            // Are render options something we need?
            // we could clone the node or and remove the attributes based on render options
            // or maybe we keep a set of attributes to ignore based on the options
            write!(w, "{}", n.to_dot_string());
        }

        for e in graph.edges {

        }

        // for e in graph.edges.iter() {
        //     let escaped_label = &graph.edge_label(e).to_dot_string();
        //     write!(w, "{}", INDENT)?;
        //     let source = graph.source(e);
        //     let target = graph.target(e);
        //     let source_id = graph.node_id(&source);
        //     let target_id = graph.node_id(&target);

        //     let mut text = Vec::new();
        //     write!(text, "{} -> {}", source_id.as_slice(), target_id.as_slice()).unwrap();

        //     if !options.contains(&RenderOption::NoEdgeLabels) {
        //         write!(text, "[label={}]", escaped_label).unwrap();
        //     }

        //     let style = graph.edge_style(e);
        //     if !options.contains(&RenderOption::NoEdgeStyles) && style != Style::None {
        //         write!(text, "[style=\"{}\"]", style.as_slice()).unwrap();
        //     }

        //     writeln!(text, ";").unwrap();
        //     w.write_all(&text[..])?;
        // }

        writeln!(w, "}}")
    }
}

// impl fmt::Display for Dot {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.render(&self.graph, f)
//     }
// }

// impl fmt::Debug for Dot {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.render(&self.graph, f)
//     }
// }

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


// TODO: better name for this trait?
pub trait GraphTraits {

    /// Add a general or graph/node/edge attribute statement.
    /// ``None`` or ``'graph'``, ``'node'``, ``'edge'`
    fn add_attribute();

    fn add_node(label: &str);

    // fn add_edge(a: NodeIndex, b: NodeIndex, label: &str);

    // <(), i32>
    /// deps.extend_with_edges(&[
    ///     (pg, fb), (pg, qc),
    ///     (qc, rand), (rand, libc), (qc, libc),
    /// ]);
    /// pub fn from_edges<I>(iterable: I) -> Self
    fn add_edges();

    fn add_subgraph();
}


/// Marker type for a directed graph.
#[derive(Clone, Copy, Debug)]
pub enum Directed {}

/// Marker type for an undirected graph.
#[derive(Clone, Copy, Debug)]
pub enum Undirected {}


/// A graph's edge type determines whether it has directed edges or not.
pub trait EdgeType {
    fn is_directed() -> bool;

    // TODO: maybe this doesnt below here
    fn as_slice() -> &'static str;
}

impl EdgeType for Directed {
    fn is_directed() -> bool {
        true
    }

    fn as_slice() -> &'static str {
        "->"
    }
}

impl EdgeType for Undirected {
    fn is_directed() -> bool {
        false
    }

    fn as_slice() -> &'static str {
        "--"
    }
}

pub type DiGraph<'a> = Graph<'a, Directed>;

pub type UnGraph<'a> = Graph<'a, Undirected>;

pub struct Graph<'a, Ty = Directed> {

    pub id: Option<String>,
    
    pub strict: bool,

    // Comment added to the first line of the source.
    pub comment: Option<String>,

    pub graph_attributes: Option<Vec<String>>,

    pub nodes: Vec<Node<'a>>,

    pub edges: Vec<String>,

    ty: PhantomData<Ty>,

    // TODO: should this have
    // pub graph_type: Ty,
    // then have Directed and Undirected enums implement fn to print graph type string?
    // pub graph_type: Ty,
}

impl<'a> Graph<'a, Directed> {
    pub fn new() -> Self {
        Graph {
            id: None,
            strict: false,
            comment: None,
            graph_attributes: None,
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
    pub fn new_undirected() -> Self {
        Graph {
            id: None,
            strict: false,
            comment: None,
            graph_attributes: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            ty: PhantomData,
        }
    }
}

impl<'a, Ty> Graph<'a, Ty> 
where Ty: EdgeType
{
    /// Whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    pub fn edge_type(&self) -> &'static str {
        Ty::as_slice()
    }
}

pub struct UndirectedGraph<'a> {

    pub id: Option<String>,

    pub strict: bool,

    // Comment added to the first line of the source.
    pub comment: String,

    pub graph_attributes: Option<Vec<String>>,

    pub nodes: Vec<Node<'a>>,

    pub edges: Vec<String>,

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
        Node {
            id: id,
            port: None,
            // compass: None,
            // shape: None,
            // label: None,
            attributes: HashMap::new(),
        }
    }

    /// Set the port for the node.
    pub fn port(&'a mut self, port: String) -> &'a mut Node {
        self.port = Some(port);
        self
    }

    // pub fn compass<'a>(&'a mut self, compass: Compass) -> &'a mut Node {
    //     self.compass = Some(compass);
    //     self
    // }

    pub fn label(&'a mut self, text: String) -> &'a mut Node {
        // self.label = Some(text);
        self.attributes.insert("label".to_string(), QuottedStr(text.into()));
        self
    }

    // TODO: create enum for shape at some point
    // pub fn shape<'a>(&'a mut self, shape: String) -> &'a mut Node {
    //     self.attributes.insert("shape".to_string(), shape);
    //     self
    // }

    /// Add an attribute to the node.
    pub fn attribute(&'a mut self, key: String, value: AttributeText<'a>) -> &'a mut Node {
        self.attributes.insert(key, value);
        self
    }

    /// Add multiple attribures to the node.
    pub fn attributes(&'a mut self, attributes: HashMap<String, AttributeText<'a>>) -> &'a mut Node {
        self.attributes.extend(attributes);
        self
    }

    pub fn to_dot_string(&self) -> String {
        let mut dot_string = format!("{}{}", INDENT, &self.id);
        // TODO: I dont love this logic. I would like to find away to not have a special case.
        // I think we introduce a AttributeText enum which encodes how to write out the attribute value
        if !self.attributes.is_empty() {
            dot_string.push_str(" [");
            for (key, value) in &self.attributes {
                dot_string.push_str(format!("{}=\"{}\"", key, value.to_dot_string()).as_str());
            }
            dot_string.push_str("];")
        }

        return dot_string.to_string();
    }

    // /// Renders text as string suitable for a label in a .dot file.
    // /// This includes quotes or suitable delimiters.
    // pub fn to_dot_string(&self) -> String {
    //     match *self {
    //         LabelStr(ref s) => format!("\"{}\"", s.escape_default()),
    //         EscStr(ref s) =>  format!("\"{}\"", LabelText::escape_str(&s)),
    //         HtmlStr(ref s) => format!("<{}>", s),
    //     }
    // }

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

#[test]
fn empty_digraph() {
    let g = Graph::new();
    //let mut writer = Vec::new();
    //let dot = Dot::
//     let r = test_input(dot);
//     assert_eq!(
//         r.unwrap(),
//         r#"digraph empty_graph {
// }
// "#
//     );
}