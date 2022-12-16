use crate::vm::flowgraph::{Color, Edge, EdgeStyle, EdgeType, Node, NodeShape};

/// This represents the direction of flow in the flowgraph.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Represents a top to bottom direction.
    TopToBottom,

    /// Represents a bottom to top direction.
    BottomToTop,

    /// Represents a left to right direction.
    LeftToRight,

    /// Represents a right to left direction.
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
    #[inline]
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
    #[inline]
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// Set the direction of the subgraph.
    #[inline]
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Add a node to the subgraph.
    #[inline]
    pub fn add_node(&mut self, location: usize, shape: NodeShape, label: Box<str>, color: Color) {
        let node = Node::new(location, shape, label, color);
        self.nodes.push(node);
    }

    /// Add an edge to the subgraph.
    #[inline]
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
    #[inline]
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
    #[inline]
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
            let color = format!(",style=filled,color=\"{}\"", node.color);
            result.push_str(&format!(
                "\t\t{prefix}_{}_i_{}[label=\"{:04}: {}\"{shape}{color}];\n",
                self.label, node.location, node.location, node.label
            ));
        }

        for edge in &self.edges {
            let color = format!(",color=\"{}\"", edge.color);
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
    #[inline]
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
        result.push_str(&format!("  direction {rankdir}\n"));

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
            let (shape_begin, shape_end) = match node.shape {
                NodeShape::None | NodeShape::Record => ('[', ']'),
                NodeShape::Diamond => ('{', '}'),
            };
            result.push_str(&format!(
                "  {prefix}_{}_i_{}{shape_begin}\"{:04}: {}\"{shape_end}\n",
                self.label, node.location, node.location, node.label
            ));
            if !node.color.is_none() {
                result.push_str(&format!(
                    "  style {prefix}_{}_i_{} fill:{}\n",
                    self.label, node.location, node.color
                ));
            }
        }

        for (index, edge) in self.edges.iter().enumerate() {
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

            if !edge.color.is_none() {
                result.push_str(&format!(
                    "  linkStyle {} stroke:{}, stroke-width: 4px\n",
                    index + 1,
                    edge.color
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
    #[inline]
    #[must_use]
    pub fn new(direction: Direction) -> Self {
        Graph {
            subgraphs: Vec::default(),
            direction,
        }
    }

    /// Create a [`SubGraph`] in this [`Graph`].
    #[inline]
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
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
    pub fn to_mermaid_format(&self) -> String {
        let mut result = String::new();
        let rankdir = match self.direction {
            Direction::TopToBottom => "TD",
            Direction::BottomToTop => "DT",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result += &format!("graph {rankdir}\n");

        for subgraph in &self.subgraphs {
            subgraph.mermaid_format(&mut result, "");
        }
        result += "\n";
        result
    }
}
