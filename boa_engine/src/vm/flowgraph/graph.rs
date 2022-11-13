use crate::vm::flowgraph::{Color, Edge, EdgeStyle, EdgeType, Node, NodeShape};

/// This represents the direction of flow in the flowgraph.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

/// Represents a sub-graph in the graph.
///
/// Sub-graphs can be nested.
#[derive(Debug, Clone)]
pub struct SubGraph {
    /// The label on the sub-graph.
    label: String,
    /// The nodes it contains.
    nodes: Vec<Node>,
    /// The edges/connections in contains.
    edges: Vec<Edge>,
    /// The direction of flow in the sub-graph.
    direction: Direction,

    /// The sub-graphs this graph contains.
    subgraphs: Vec<SubGraph>,
}

impl SubGraph {
    /// Construct a new subgraph.
    fn new(label: String) -> Self {
        Self {
            label,
            nodes: Vec::default(),
            edges: Vec::default(),
            direction: Direction::TopToBottom,
            subgraphs: Vec::default(),
        }
    }

    /// Set the label of the subgraph.
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// Set the direction of the subgraph.
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Add a node to the subgraph.
    pub fn add_node(&mut self, location: usize, shape: NodeShape, label: Box<str>, color: Color) {
        let node = Node::new(location, shape, label, color);
        self.nodes.push(node);
    }

    /// Add an edge to the subgraph.
    pub fn add_edge(
        &mut self,
        from: usize,
        to: usize,
        label: Option<Box<str>>,
        color: Color,
        style: EdgeStyle,
    ) -> &mut Edge {
        let edge = Edge::new(from, to, label, color, style);
        self.edges.push(edge);
        self.edges.last_mut().expect("Already pushed edge")
    }

    /// Create a subgraph in this subgraph.
    pub fn subgraph(&mut self, label: String) -> &mut SubGraph {
        self.subgraphs.push(SubGraph::new(label));
        let result = self
            .subgraphs
            .last_mut()
            .expect("We just pushed a subgraph");
        result.set_direction(self.direction);
        result
    }

    /// Format into the graphviz format.
    fn graphviz_format(&self, result: &mut String, prefix: &str) {
        result.push_str(&format!("\tsubgraph cluster_{prefix}_{} {{\n", self.label));
        result.push_str("\t\tstyle = filled;\n");
        result.push_str(&format!(
            "\t\tlabel = \"{}\";\n",
            if self.label.is_empty() {
                "Anonymous Function"
            } else {
                self.label.as_ref()
            }
        ));

        result.push_str(&format!(
            "\t\t{prefix}_{}_start [label=\"Start\",shape=Mdiamond,style=filled,color=green]\n",
            self.label
        ));
        if !self.nodes.is_empty() {
            result.push_str(&format!(
                "\t\t{prefix}_{}_start -> {prefix}_{}_i_0\n",
                self.label, self.label
            ));
        }

        for node in &self.nodes {
            let shape = match node.shape {
                NodeShape::None => "",
                NodeShape::Record => ", shape=record",
                NodeShape::Diamond => ", shape=diamond",
            };
            let color = match node.color {
                Color::None => String::new(),
                Color::Red => ",style=filled,color=red".into(),
                Color::Green => ",style=filled,color=green".into(),
                Color::Blue => ",style=filled,color=blue".into(),
                Color::Yellow => ",style=filled,color=yellow".into(),
                Color::Purple => ",style=filled,color=purple".into(),
                Color::Rgb(color) => format!(",style=filled,color=\"#{color:X}\""),
            };
            result.push_str(&format!(
                "\t\t{prefix}_{}_i_{}[label=\"{:04}: {}\"{shape}{color}];\n",
                self.label, node.location, node.location, node.label
            ));
        }

        for edge in &self.edges {
            let color = match edge.color {
                Color::None => String::new(),
                Color::Red => ",color=red".into(),
                Color::Green => ",color=green".into(),
                Color::Blue => ",color=blue".into(),
                Color::Yellow => ",color=yellow".into(),
                Color::Purple => ",color=purple".into(),
                Color::Rgb(color) => format!(",color=\"#{color:X}\""),
            };
            let style = match (edge.style, edge.type_) {
                (EdgeStyle::Line, EdgeType::None) => ",dir=none",
                (EdgeStyle::Line, EdgeType::Arrow) => "",
                (EdgeStyle::Dotted, EdgeType::None) => ",style=dotted,dir=none",
                (EdgeStyle::Dotted, EdgeType::Arrow) => ",style=dotted",
                (EdgeStyle::Dashed, EdgeType::None) => ",style=dashed,dir=none",
                (EdgeStyle::Dashed, EdgeType::Arrow) => ",style=dashed,",
            };
            result.push_str(&format!(
                "\t\t{prefix}_{}_i_{} -> {prefix}_{}_i_{} [label=\"{}\", len=f{style}{color}];\n",
                self.label,
                edge.from,
                self.label,
                edge.to,
                edge.label.as_deref().unwrap_or("")
            ));
        }
        for (index, subgraph) in self.subgraphs.iter().enumerate() {
            let prefix = format!("{prefix}_F{index}");
            subgraph.graphviz_format(result, &prefix);
        }
        result.push_str("\t}\n");
    }

    /// Format into the mermaid format.
    fn mermaid_format(&self, result: &mut String, prefix: &str) {
        let rankdir = match self.direction {
            Direction::TopToBottom => "TB",
            Direction::BottomToTop => "BT",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result.push_str(&format!(
            "  subgraph {prefix}_{}[\"{}\"]\n",
            self.label,
            if self.label.is_empty() {
                "Anonymous Function"
            } else {
                self.label.as_ref()
            }
        ));
        result.push_str(&format!("  direction {}\n", rankdir));

        result.push_str(&format!("  {prefix}_{}_start{{Start}}\n", self.label));
        result.push_str(&format!(
            "  style {prefix}_{}_start fill:green\n",
            self.label
        ));
        if !self.nodes.is_empty() {
            result.push_str(&format!(
                "  {prefix}_{}_start --> {prefix}_{}_i_0\n",
                self.label, self.label
            ));
        }

        for node in &self.nodes {
            let color = match node.color {
                Color::None => String::new(),
                Color::Red => "red".into(),
                Color::Green => "green".into(),
                Color::Blue => "blue".into(),
                Color::Yellow => "yellow".into(),
                Color::Purple => "purple".into(),
                Color::Rgb(color) => format!("#{color:X}"),
            };
            let (shape_begin, shape_end) = match node.shape {
                NodeShape::None | NodeShape::Record => ('[', ']'),
                NodeShape::Diamond => ('{', '}'),
            };
            result.push_str(&format!(
                "  {prefix}_{}_i_{}{shape_begin}\"{:04}: {}\"{shape_end}\n",
                self.label, node.location, node.location, node.label
            ));
            if !color.is_empty() {
                result.push_str(&format!(
                    "  style {prefix}_{}_i_{} fill:{color}\n",
                    self.label, node.location
                ));
            }
        }

        for (index, edge) in self.edges.iter().enumerate() {
            let color = match edge.color {
                Color::None => String::new(),
                Color::Red => "red".into(),
                Color::Green => "green".into(),
                Color::Blue => "blue".into(),
                Color::Yellow => "yellow".into(),
                Color::Purple => "purple".into(),
                Color::Rgb(color) => format!("#{color:X}"),
            };
            let style = match (edge.style, edge.type_) {
                (EdgeStyle::Line, EdgeType::None) => "---",
                (EdgeStyle::Line, EdgeType::Arrow) => "-->",
                (EdgeStyle::Dotted | EdgeStyle::Dashed, EdgeType::None) => "-.-",
                (EdgeStyle::Dotted | EdgeStyle::Dashed, EdgeType::Arrow) => "-.->",
            };
            result.push_str(&format!(
                "  {prefix}_{}_i_{} {style}| {}| {prefix}_{}_i_{}\n",
                self.label,
                edge.from,
                edge.label.as_deref().unwrap_or(""),
                self.label,
                edge.to,
            ));

            if !color.is_empty() {
                result.push_str(&format!(
                    "  linkStyle {} stroke:{}, stroke-width: 4px\n",
                    index + 1,
                    color
                ));
            }
        }
        for (index, subgraph) in self.subgraphs.iter().enumerate() {
            let prefix = format!("{prefix}_F{index}");
            subgraph.mermaid_format(result, &prefix);
        }
        result.push_str("  end\n");
    }
}

/// This represents the main graph that other [`SubGraph`]s can be nested in.
#[derive(Debug)]
pub struct Graph {
    subgraphs: Vec<SubGraph>,
    direction: Direction,
}

impl Graph {
    /// Construct a [`Graph`]
    pub fn new(direction: Direction) -> Self {
        Graph {
            subgraphs: Vec::default(),
            direction,
        }
    }

    /// Create a [`SubGraph`] in this [`Graph`].
    pub fn subgraph(&mut self, label: String) -> &mut SubGraph {
        self.subgraphs.push(SubGraph::new(label));
        let result = self
            .subgraphs
            .last_mut()
            .expect("We just pushed a subgraph");
        result.set_direction(self.direction);
        result
    }

    /// Output the graph into the graphviz format.
    pub fn to_graphviz_format(&self) -> String {
        let mut result = String::new();
        result += "digraph {\n";
        result += "\tnode [shape=record];\n";

        let rankdir = match self.direction {
            Direction::TopToBottom => "TB",
            Direction::BottomToTop => "BT",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result += &format!("\trankdir={rankdir};\n");

        for subgraph in &self.subgraphs {
            subgraph.graphviz_format(&mut result, "");
        }
        result += "}\n";
        result
    }

    /// Output the graph into the mermaid format.
    pub fn to_mermaid_format(&self) -> String {
        let mut result = String::new();
        let rankdir = match self.direction {
            Direction::TopToBottom => "TD",
            Direction::BottomToTop => "DT",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result += &format!("graph {}\n", rankdir);

        for subgraph in &self.subgraphs {
            subgraph.mermaid_format(&mut result, "");
        }
        result += "\n";
        result
    }
}
