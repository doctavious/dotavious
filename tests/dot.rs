use dotavious::attributes::{
    AttributeText, AttributeType, Color, CompassPoint, EdgeAttributes, EdgeStyle,
    GraphAttributeStatementBuilder, GraphAttributes, GraphStyle, NodeAttributes,
    NodeStyle, PortPosition, RankDir, Shape,
};
use dotavious::{
    Dot, Edge, EdgeAttributeStatementBuilder, EdgeBuilder, Graph, GraphBuilder, Node,
    NodeAttributeStatementBuilder, NodeBuilder, SubGraphBuilder,
};
use std::io;
use std::io::Read;

fn test_input(g: Graph) -> io::Result<String> {
    let mut writer = Vec::new();
    let dot = Dot { graph: g };

    dot.render(&mut writer).unwrap();

    let mut s = String::new();
    Read::read_to_string(&mut &*writer, &mut s)?;
    Ok(s)
}

#[test]
fn empty_digraph_without_id() {
    let g = GraphBuilder::new_directed(None).build();
    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"digraph {
}
"#
    );
}

#[test]
fn support_display() {
    let g = GraphBuilder::new_directed(None).build();
    let dot = Dot { graph: g };

    assert_eq!(
        format!("{}", dot),
        r#"digraph {
}
"#
    );
}

#[test]
fn graph_comment() {
    let g = GraphBuilder::new_directed(None)
        .comment("Comment goes here")
        .build();
    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"// Comment goes here
digraph {
}
"#
    );
}

#[test]
fn empty_digraph() {
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
fn empty_undirected_graph() {
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
        .style(NodeStyle::Dashed)
        .build();

    let g = GraphBuilder::new_directed(Some("single_node".to_string()))
        .add_node(node)
        .build();

    let r = test_input(g);
    assert_eq!(
        r.unwrap(),
        r#"digraph single_node {
    N0 [style=dashed];
}
"#
    );
}

#[test]
fn support_non_inline_builder() {
    let mut g = GraphBuilder::new_directed(Some("single_node".to_string()));

    // TODO: having to split this is stupid. am i doing something wrong?
    let mut node_builder = NodeBuilder::new("N0".to_string());
    node_builder.style(NodeStyle::Dashed);

    if true {
        node_builder.add_attribute("foo", AttributeText::quoted("baz"));
    }

    let node = node_builder.build();
    g.add_node(node);

    let r = test_input(g.build());
    assert_eq!(
        r.unwrap(),
        r#"digraph single_node {
    N0 [style=dashed, foo="baz"];
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
        .style(EdgeStyle::Bold)
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
    N0 -> N1 [style=bold];
}
"#
    );
}

#[test]
fn edge_statement_port_position() {
    let node_0 = NodeBuilder::new("N0".to_string())
        .shape(Shape::Record)
        .label("a|<port0>b")
        .build();

    let node_1 = NodeBuilder::new("N1".to_string())
        .shape(Shape::Record)
        .label("e|<port1>f")
        .build();

    let edge = EdgeBuilder::new("N0".to_string(), "N1".to_string())
        .source_port_position(PortPosition::Port {
            port_name: "port0".to_string(),
            compass_point: Some(CompassPoint::SW),
        })
        .target_port_position(PortPosition::Port {
            port_name: "port1".to_string(),
            compass_point: Some(CompassPoint::NE),
        })
        .build();

    let g = GraphBuilder::new_directed(Some("edge_statement_port_position".to_string()))
        .add_node(node_0)
        .add_node(node_1)
        .add_edge(edge)
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph edge_statement_port_position {
    N0 [shape=record, label="a|<port0>b"];
    N1 [shape=record, label="e|<port1>f"];
    N0:port0:sw -> N1:port1:ne;
}
"#
    );
}

#[test]
fn port_position_attribute() {
    let node_0 = NodeBuilder::new("N0".to_string())
        .shape(Shape::Record)
        .label("a|<port0>b")
        .build();

    let node_1 = NodeBuilder::new("N1".to_string())
        .shape(Shape::Record)
        .label("e|<port1>f")
        .build();

    let edge = EdgeBuilder::new("N0".to_string(), "N1".to_string())
        .tail_port(PortPosition::Port {
            port_name: "port0".to_string(),
            compass_point: Some(CompassPoint::SW),
        })
        .head_port(PortPosition::Port {
            port_name: "port1".to_string(),
            compass_point: Some(CompassPoint::NE),
        })
        .build();

    let g = GraphBuilder::new_directed(Some("port_position_attribute".to_string()))
        .add_node(node_0)
        .add_node(node_1)
        .add_edge(edge)
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph port_position_attribute {
    N0 [shape=record, label="a|<port0>b"];
    N1 [shape=record, label="e|<port1>f"];
    N0 -> N1 [tailport="port0:sw", headport="port1:ne"];
}
"#
    );
}

#[test]
fn graph_attributes() {
    let g = GraphBuilder::new_directed(Some("graph_attributes".to_string()))
        .add_attribute(
            AttributeType::Graph,
            "rankdir".to_string(),
            AttributeText::from(RankDir::LeftRight),
        )
        .add_attribute(
            AttributeType::Node,
            "style".to_string(),
            AttributeText::from(NodeStyle::Filled),
        )
        .add_attribute(
            AttributeType::Edge,
            "color".to_string(),
            AttributeText::from(Color::Named("red")),
        )
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph graph_attributes {
    graph [rankdir=LR];
    node [style=filled];
    edge [color="red"];
}
"#
    );
}

#[test]
fn graph_attributes_extend() {
    let g = GraphBuilder::new_directed(Some("graph_attributes".to_string()))
        .extend_with_attributes(
            AttributeType::Graph,
            [(
                "rankdir".to_string(),
                AttributeText::from(RankDir::LeftRight),
            )]
            .iter()
            .cloned()
            .collect(),
        )
        .extend_with_attributes(
            AttributeType::Node,
            [("style".to_string(), AttributeText::from(NodeStyle::Filled))]
                .iter()
                .cloned()
                .collect(),
        )
        .extend_with_attributes(
            AttributeType::Edge,
            [(
                "color".to_string(),
                AttributeText::from(Color::Named("red")),
            )]
            .iter()
            .cloned()
            .collect(),
        )
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph graph_attributes {
    graph [rankdir=LR];
    node [style=filled];
    edge [color="red"];
}
"#
    );
}

#[test]
fn graph_attributes_statement_builders() {
    let graph_attributes = GraphAttributeStatementBuilder::new()
        .rank_dir(RankDir::LeftRight)
        .build();
    let node_attributes = NodeAttributeStatementBuilder::new()
        .style(NodeStyle::Filled)
        .build();
    let edge_attributes = EdgeAttributeStatementBuilder::new()
        .color(Color::Named("red"))
        .build();

    let g = GraphBuilder::new_directed(Some("graph_attributes".to_string()))
        .add_graph_attributes(graph_attributes)
        .add_node_attributes(node_attributes)
        .add_edge_attributes(edge_attributes)
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph graph_attributes {
    graph [rankdir=LR];
    node [style=filled];
    edge [color="red"];
}
"#
    );
}

#[test]
fn clusters() {
    let cluster_0 = SubGraphBuilder::new(Some("cluster_0".to_string()))
        .add_graph_attributes(
            GraphAttributeStatementBuilder::new()
                .label("process #1".to_string())
                .style(GraphStyle::Filled)
                .color(Color::Named("lightgrey"))
                .build(),
        )
        .add_node_attributes(
            NodeAttributeStatementBuilder::new()
                .style(NodeStyle::Filled)
                .color(Color::Named("white"))
                .build(),
        )
        .add_edge(Edge::new("a0".to_string(), "a1".to_string()))
        .add_edge(Edge::new("a1".to_string(), "a2".to_string()))
        .add_edge(Edge::new("a2".to_string(), "a3".to_string()))
        .build();

    let cluster_1 = SubGraphBuilder::new(Some("cluster_1".to_string()))
        .add_graph_attributes(
            GraphAttributeStatementBuilder::new()
                .label("process #2".to_string())
                .style(GraphStyle::Filled)
                .color(Color::Named("blue"))
                .build(),
        )
        .add_node_attributes(
            NodeAttributeStatementBuilder::new()
                .style(NodeStyle::Filled)
                .build(),
        )
        .add_edge(Edge::new("b0".to_string(), "b1".to_string()))
        .add_edge(Edge::new("b1".to_string(), "b2".to_string()))
        .add_edge(Edge::new("b2".to_string(), "b3".to_string()))
        .build();

    let g = GraphBuilder::new_directed(Some("G".to_string()))
        .add_node(
            NodeBuilder::new("start".to_string())
                .shape(Shape::Mdiamond)
                .build(),
        )
        .add_node(
            NodeBuilder::new("end".to_string())
                .shape(Shape::Msquare)
                .build(),
        )
        .add_sub_graph(cluster_0)
        .add_sub_graph(cluster_1)
        .add_edge(Edge::new("start".to_string(), "a0".to_string()))
        .add_edge(Edge::new("start".to_string(), "b0".to_string()))
        .add_edge(Edge::new("a1".to_string(), "b3".to_string()))
        .add_edge(Edge::new("b2".to_string(), "a3".to_string()))
        .add_edge(Edge::new("a3".to_string(), "a0".to_string()))
        .add_edge(Edge::new("a3".to_string(), "end".to_string()))
        .add_edge(Edge::new("b3".to_string(), "end".to_string()))
        .build();

    let r = test_input(g);

    assert_eq!(
        r.unwrap(),
        r#"digraph G {
    subgraph cluster_0 {
        graph [label="process #1", style=filled, color="lightgrey"];
        node [style=filled, color="white"];
        a0 -> a1;
        a1 -> a2;
        a2 -> a3;
    }

    subgraph cluster_1 {
        graph [label="process #2", style=filled, color="blue"];
        node [style=filled];
        b0 -> b1;
        b1 -> b2;
        b2 -> b3;
    }

    start [shape=Mdiamond];
    end [shape=Msquare];
    start -> a0;
    start -> b0;
    a1 -> b3;
    b2 -> a3;
    a3 -> a0;
    a3 -> end;
    b3 -> end;
}
"#
    );
}
