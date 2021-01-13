// TODO: docs

use crate::attributes::{
    fmt_attributes, AttributeStatement, AttributeText, AttributeType, EdgeAttributes,
    GraphAttributeStatement, NodeAttributes, PortPosition,
};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::prelude::*;

static INDENT: &str = "    ";

pub trait DotString<'a> {
    fn dot_string(&self) -> Cow<'a, str>;
}

pub struct Dot<'a> {
    pub graph: Graph<'a>,
}

impl<'a> Dot<'a> {
    /// Renders graph into the writer `w` in DOT syntax.
    pub fn render<W>(self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.internal_render(&self.graph, w)
    }

    fn internal_render<W>(&self, graph: &Graph, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        if let Some(comment) = &graph.comment {
            // TODO: split comment into lines of 80 or so characters
            writeln!(w, "// {}", comment)?;
        }

        let edge_op = graph.edge_op();
        let strict = if graph.strict { "strict " } else { "" };
        write!(w, "{}{}", strict, &graph.graph_type())?;

        if let Some(id) = &graph.id {
            write!(w, " {}", id)?;
        }

        writeln!(w, " {{")?;

        if let Some(graph_attributes) = &graph.graph_attributes {
            write!(w, "{}{}\n", INDENT, graph_attributes.dot_string())?;
        }

        if let Some(node_attributes) = &graph.node_attributes {
            write!(w, "{}{}\n", INDENT, node_attributes.dot_string())?;
        }

        if let Some(edge_attributes) = &graph.edge_attributes {
            write!(w, "{}{}\n", INDENT, edge_attributes.dot_string())?;
        }

        for g in &graph.sub_graphs {
            self.render_subgraph(w, g, edge_op, 1)?;
        }

        for n in &graph.nodes {
            writeln!(w, "{}{}", INDENT, n.dot_string())?;
        }

        for e in graph.edges.iter() {
            self.render_edge(w, e, edge_op, 1)?;
        }

        writeln!(w, "}}")
    }

    fn render_subgraph<W>(
        &self,
        w: &mut W,
        sub_graph: &SubGraph,
        edge_op: &str,
        indentation_level: usize,
    ) -> io::Result<()>
    where
        W: Write,
    {
        write!(w, "{}subgraph", get_indentation(indentation_level))?;
        if let Some(id) = &sub_graph.id {
            write!(w, " {}", id)?;
        }

        writeln!(w, " {{")?;

        let indent = get_indentation(indentation_level + 1);
        if let Some(graph_attributes) = &sub_graph.graph_attributes {
            write!(w, "{}{}\n", indent, graph_attributes.dot_string())?;
        }

        if let Some(node_attributes) = &sub_graph.node_attributes {
            write!(w, "{}{}\n", indent, node_attributes.dot_string())?;
        }

        if let Some(edge_attributes) = &sub_graph.edge_attributes {
            write!(w, "{}{}\n", indent, edge_attributes.dot_string())?;
        }

        for g in &sub_graph.sub_graphs {
            self.render_subgraph(w, g, edge_op, indentation_level + 1)?;
        }

        for n in &sub_graph.nodes {
            writeln!(w, "{}{}", indent, n.dot_string())?;
        }

        for e in sub_graph.edges.iter() {
            self.render_edge(w, e, edge_op, indentation_level + 1)?;
        }

        writeln!(w, "{}}}\n", get_indentation(indentation_level))
    }

    fn render_edge<W>(
        &self,
        w: &mut W,
        edge: &Edge,
        edge_op: &str,
        indentation_level: usize,
    ) -> io::Result<()>
    where
        W: Write,
    {
        let mut edge_source = edge.source.to_owned();
        if let Some(source_port_position) = &edge.source_port_position {
            edge_source
                .push_str(format!(":{}", source_port_position.dot_string()).as_str())
        }

        let mut edge_target = edge.target.to_owned();
        if let Some(target_port_position) = &edge.target_port_position {
            edge_target
                .push_str(format!(":{}", target_port_position.dot_string()).as_str())
        }

        write!(
            w,
            "{}{} {} {}",
            get_indentation(indentation_level),
            edge_source,
            edge_op,
            edge_target
        )?;
        write!(w, "{}", fmt_attributes(&edge.attributes))?;
        writeln!(w, ";")
    }
}

impl<'a> Display for Dot<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut writer = Vec::new();
        self.internal_render(&self.graph, &mut writer).unwrap();

        let mut s = String::new();
        Read::read_to_string(&mut &*writer, &mut s).unwrap();

        write!(f, "{}", s)?;

        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RenderOption {
    NoEdgeLabels,
    NoNodeLabels,
    NoEdgeStyles,
    NoNodeStyles,
    /// Use indices for node labels.
    NodeIndexLabel,
    /// Use indices for edge labels.
    EdgeIndexLabel,
}

#[derive(Clone, Debug)]
pub struct Graph<'a> {
    pub id: Option<String>,

    pub is_directed: bool,

    pub strict: bool,

    /// Comment added to the first line of the source.
    pub comment: Option<String>,

    pub graph_attributes: Option<GraphAttributeStatement<'a>>,

    pub node_attributes: Option<NodeAttributeStatement<'a>>,

    pub edge_attributes: Option<EdgeAttributeStatement<'a>>,

    pub sub_graphs: Vec<SubGraph<'a>>,

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
        sub_graphs: Vec<SubGraph<'a>>,
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
            sub_graphs,
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

    sub_graphs: Vec<SubGraph<'a>>,

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
            sub_graphs: Vec::new(),
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
            sub_graphs: Vec::new(),
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

    pub fn add_sub_graph(&mut self, sub_graph: SubGraph<'a>) -> &mut Self {
        self.sub_graphs.push(sub_graph);
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
            sub_graphs: self.sub_graphs.clone(),
            nodes: self.nodes.clone(), // TODO: is clone the only option here?
            edges: self.edges.clone(), // TODO: is clone the only option here?
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubGraph<'a> {
    pub id: Option<String>,

    pub graph_attributes: Option<GraphAttributeStatement<'a>>,

    pub node_attributes: Option<NodeAttributeStatement<'a>>,

    pub edge_attributes: Option<EdgeAttributeStatement<'a>>,

    pub sub_graphs: Vec<SubGraph<'a>>,

    pub nodes: Vec<Node<'a>>,

    pub edges: Vec<Edge<'a>>,
}

impl<'a> SubGraph<'a> {
    pub fn new(
        id: Option<String>,
        graph_attributes: Option<GraphAttributeStatement<'a>>,
        node_attributes: Option<NodeAttributeStatement<'a>>,
        edge_attributes: Option<EdgeAttributeStatement<'a>>,
        sub_graphs: Vec<SubGraph<'a>>,
        nodes: Vec<Node<'a>>,
        edges: Vec<Edge<'a>>,
    ) -> Self {
        Self {
            id,
            graph_attributes,
            node_attributes,
            edge_attributes,
            sub_graphs,
            nodes,
            edges,
        }
    }
}

pub struct SubGraphBuilder<'a> {
    id: Option<String>,

    graph_attributes: Option<GraphAttributeStatement<'a>>,

    node_attributes: Option<NodeAttributeStatement<'a>>,

    edge_attributes: Option<EdgeAttributeStatement<'a>>,

    sub_graphs: Vec<SubGraph<'a>>,

    nodes: Vec<Node<'a>>,

    edges: Vec<Edge<'a>>,
}

// TODO: id should be an escString
impl<'a> SubGraphBuilder<'a> {
    pub fn new(id: Option<String>) -> Self {
        Self {
            id,
            graph_attributes: None,
            node_attributes: None,
            edge_attributes: None,
            sub_graphs: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
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

    pub fn add_sub_graph(&mut self, sub_graph: SubGraph<'a>) -> &mut Self {
        self.sub_graphs.push(sub_graph);
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

    pub fn build(&self) -> SubGraph<'a> {
        SubGraph {
            id: self.id.to_owned(),
            graph_attributes: self.graph_attributes.clone(),
            node_attributes: self.node_attributes.clone(),
            edge_attributes: self.edge_attributes.clone(),
            sub_graphs: self.sub_graphs.clone(),
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
        dot_string.push_str(fmt_attributes(&self.attributes).as_str());
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

    /// Add multiple attributes to the edge.
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

fn get_indentation(indentation_level: usize) -> String {
    INDENT.repeat(indentation_level)
}
