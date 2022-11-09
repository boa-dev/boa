#[derive(Debug, Clone, Copy)]
pub enum NodeShape {
    None,
    Record,
    Diamond,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeColor {
    None,
}

#[derive(Debug)]
pub struct Node {
    location: usize,
    shape: NodeShape,
    label: Box<str>,
    color: NodeColor,
}

impl Node {
    pub fn new(location: usize, shape: NodeShape, label: Box<str>, color: NodeColor) -> Self {
        Self {
            location,
            shape,
            label,
            color,
        }
    }
    pub fn location(&self) -> usize {
        self.location
    }
}

#[derive(Debug)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: Option<Box<str>>,
    pub color: NodeColor,
}

impl Edge {
    pub fn new(from: usize, to: usize, label: Option<Box<str>>, color: NodeColor) -> Self {
        Self {
            from,
            to,
            label,
            color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RankDirection {
    TopToBottom,
    LeftToRight,
    RightToLeft,
}

#[derive(Debug)]
pub struct Graph {
    label: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    rank_direction: RankDirection,
}

impl Graph {
    pub fn new(label: String) -> Self {
        Self {
            label,
            nodes: Vec::default(),
            edges: Vec::default(),
            rank_direction: RankDirection::TopToBottom,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn to_graphviz_format(&self) -> String {
        let mut result = String::new();
        result += "digraph {\n";
        result += "\tnode [shape=record];\n";

        let rankdir = match self.rank_direction {
            RankDirection::TopToBottom => "TB",
            RankDirection::LeftToRight => "LR",
            RankDirection::RightToLeft => "RL",
        };
        result += &format!("\trankdir={rankdir};\n");

        result += &format!("\tsubgraph cluster_{} {{\n", self.label);
        result += "\t\tstyle = filled;\n";
        result += &format!("\t\tlabel = \"{}\";\n", self.label);
        for node in &self.nodes {
            let shape = match node.shape {
                NodeShape::None => "",
                NodeShape::Record => ", shape=record",
                NodeShape::Diamond => ", shape=diamond",
            };
            let color = match node.color {
                NodeColor::None => "",
            };
            result += &format!(
                "\t\t{}_i_{}[label=\"{}\"{shape}{color}];\n",
                self.label, node.location, node.label
            );
        }

        for edge in &self.edges {
            let color = match edge.color {
                NodeColor::None => "",
            };
            result += &format!(
                "\t\t{}_i_{} -> {}_i_{} [label=\"{}\", len=f {color}];\n",
                self.label,
                edge.from,
                self.label,
                edge.to,
                edge.label.as_deref().unwrap_or("")
            );
        }
        result += "\t}\n";
        result += "}\n";
        result
    }

    pub fn to_mermaid_format(&self) -> String {
        let mut result = String::new();
        let rankdir = match self.rank_direction {
            RankDirection::TopToBottom => "TD",
            RankDirection::LeftToRight => "LR",
            RankDirection::RightToLeft => "RL",
        };
        result += &format!("graph {}\n", rankdir);

        for node in &self.nodes {
            let (shape_begin, shape_end) = match node.shape {
                NodeShape::None | NodeShape::Record => ('[', ']'),
                NodeShape::Diamond => ('{', '}'),
            };
            result += &format!(
                "    {}_i_{}{shape_begin}{}{shape_end}\n",
                self.label, node.location, node.label
            );
        }

        for edge in &self.edges {
            let color = match edge.color {
                NodeColor::None => "",
            };
            result += &format!(
                "    {}_i_{} -->| {}| {}_i_{}\n",
                self.label,
                edge.from,
                edge.label.as_deref().unwrap_or(""),
                self.label,
                edge.to,
            );
        }
        result += "\n";
        result
    }
}
