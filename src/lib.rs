//! Simple graphviz dot file format output.

use std;
use std::collections::HashMap;
use std::io;
use std::io::{
    Write
};
use std::marker::PhantomData;

static INDENT: &str = "    ";

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

pub struct Dot {
    graph: Graph,
    config: Config,
}

impl Dot {

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
    pub fn render_opts<'a, W>(self, graph: Graph, w: &mut W, options: &[RenderOption]) -> io::Result<()>
    where
        W: Write,
    {
        writeln!(w, "{}", graph.comment.unwrap_or_default())?;

        let strict = if graph.strict { "strict " } else { "" }; 

        // TODO: can we use unwrap_or_default?
        let id = match graph.id {
            Some(v) => v,
            None => String::new()
        };

        // TODO: implement
        // writeln!(w, "{}{} {}{{", strict, graph.graph_type.as_slice(), id)?;

        // Global graph properties
        if options.contains(&RenderOption::Monospace) {
            writeln!(w, r#"    graph[fontname="monospace"];"#)?;
            writeln!(w, r#"    node[fontname="monospace"];"#)?;
            writeln!(w, r#"    edge[fontname="monospace"];"#)?;
        }

        for n in graph.nodes {
            write!(w, "{}", INDENT)?;
            // let id = graph.node_id(n);
            // let id = n.id;

            let mut text = Vec::new();
            write!(text, "{}", n.id).unwrap();

            if !options.contains(&RenderOption::NoNodeLabels) {
                // TODO: implement
                // let label = &graph.node_label(n).to_dot_string();
                let label = "";
                write!(text, "[label={}]", label).unwrap();
            }

            // TODO: implement
            // let style = graph.node_style(n);
            // let style = n.style;
            // if !options.contains(&RenderOption::NoNodeStyles) && style != Style::None {
            //     write!(text, "[style=\"{}\"]", style.as_slice()).unwrap();
            // }

            // if let Some(s) = graph.node_shape(n) {
            //     write!(text, "[shape={}]", &s.to_dot_string()).unwrap();
            // }
            // if let Some(s) = n.shape {
            //     TODO: implement
            //     write!(text, "[shape={}]", &s.to_dot_string()).unwrap();
            // }

            writeln!(text, ";").unwrap();
            w.write_all(&text[..])?;
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

pub type DiGraph = Graph<Directed>;

pub type UnGraph = Graph<Undirected>;

pub struct Graph<Ty = Directed> {

    pub id: Option<String>,
    
    pub strict: bool,

    // Comment added to the first line of the source.
    pub comment: Option<String>,

    pub graph_attributes: Option<Vec<String>>,

    pub nodes: Vec<Node>,

    pub edges: Vec<String>,

    ty: PhantomData<Ty>,

    // TODO: should this have
    // pub graph_type: Ty,
    // then have Directed and Undirected enums implement fn to print graph type string?
    // pub graph_type: Ty,
}

impl Graph<Directed> {
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

impl Graph<Undirected> {
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

impl<Ty> Graph<Ty> 
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

pub struct UndirectedGraph {

    pub id: Option<String>,

    pub strict: bool,

    // Comment added to the first line of the source.
    pub comment: String,

    pub graph_attributes: Option<Vec<String>>,

    pub nodes: Vec<Node>,

    pub edges: Vec<String>,

}


// TODO: add node builder using "with" convention
pub struct Node {

    pub id: String,

    pub port: Option<String>,

    // pub compass: Option<Compass>,

    // // TODO: enum?
    // pub shape: Option<String>,
    
    pub attributes: HashMap<String, String>,

    // style

}

impl Node {

    pub fn new(id: String) -> Node {
        Node {
            id: id,
            port: None,
            // compass: None,
            // shape: None,
            attributes: HashMap::new(),
        }
    }

    /// Set the port for the node.
    pub fn port<'a>(&'a mut self, port: String) -> &'a mut Node {
        self.port = Some(port);
        self
    }

    // pub fn compass<'a>(&'a mut self, compass: Compass) -> &'a mut Node {
    //     self.compass = Some(compass);
    //     self
    // }

    pub fn label<'a>(&'a mut self, text: String) -> &'a mut Node {
        self.attributes.insert("label".to_string(), text);
        self
    }

    // TODO: create enum for shape at some point
    pub fn shape<'a>(&'a mut self, shape: String) -> &'a mut Node {
        self.attributes.insert("shape".to_string(), shape);
        self
    }

    /// Add an attribute to the node.
    pub fn attribute<'a>(&'a mut self, key: String, value: String) -> &'a mut Node {
        self.attributes.insert(key, value);
        self
    }

    /// Add multiple attribures to the node.
    pub fn attributes<'a>(&'a mut self, attributes: HashMap<String, String>) -> &'a mut Node {
        self.attributes.extend(attributes);
        self
    }

    pub fn to_dot_string(&self) -> String {
        let mut dot_string = String::from(&self.id);
        if !self.attributes.is_empty() {
            dot_string.push_str("[");
            for (key, value) in &self.attributes {
                // TODO: most attributes outside of label dont need wrapping quotes
                // but right now label is in the dictionary with the other attributes
                // We can split it out
                dot_string.push_str(format!("{}=\"{}\"", key, value));
            }
            dot_string.push_str("]")
        }

        return dot_string.to_string();
    }

    // /// Renders text as string suitable for a label in a .dot file.
    // /// This includes quotes or suitable delimiters.
    // pub fn to_dot_string(&self) -> String {
    //     match *self {
    //         LabelStr(ref s) => format!("\"{}\"", s.escape_default()),
    //         EscStr(ref s) => format!("\"{}\"", LabelText::escape_str(&s)),
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