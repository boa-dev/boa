#[derive(Debug, Clone, Copy)]
pub enum NodeShape {
    None,
    Record,
    Diamond,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    None,
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Color(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeStyle {
    Line,
    Dotted,
    Dashed,
}

#[derive(Debug)]
pub struct Node {
    location: usize,
    shape: NodeShape,
    label: Box<str>,
    color: Color,
}

impl Node {
    pub fn new(location: usize, shape: NodeShape, label: Box<str>, color: Color) -> Self {
        Self {
            location,
            shape,
            label,
            color,
        }
    }
}

#[derive(Debug)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: Option<Box<str>>,
    pub color: Color,
    pub style: EdgeStyle,
}

impl Edge {
    pub fn new(
        from: usize,
        to: usize,
        label: Option<Box<str>>,
        color: Color,
        style: EdgeStyle,
    ) -> Self {
        Self {
            from,
            to,
            label,
            color,
            style,
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

    pub fn add_node(&mut self, location: usize, shape: NodeShape, label: Box<str>, color: Color) {
        let node = Node::new(location, shape, label, color);
        self.nodes.push(node);
    }

    pub fn add_edge(
        &mut self,
        from: usize,
        to: usize,
        label: Option<Box<str>>,
        color: Color,
        style: EdgeStyle,
    ) {
        let edge = Edge::new(from, to, label, color, style);
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
                Color::None => String::new(),
                Color::Red => ",style=filled,color=red".into(),
                Color::Green => ",style=filled,color=green".into(),
                Color::Blue => ",style=filled,color=blue".into(),
                Color::Yellow => ",style=filled,color=yellow".into(),
                Color::Purple => ",style=filled,color=purple".into(),
                Color::Color(color) => format!(",style=filled,color=\\\"#{color:X}\\\""),
            };
            result += &format!(
                "\t\t{}_i_{}[label=\"{}\"{shape}{color}];\n",
                self.label, node.location, node.label
            );
        }

        for edge in &self.edges {
            let color = match edge.color {
                Color::None => String::new(),
                Color::Red => ",color=red".into(),
                Color::Green => ",color=green".into(),
                Color::Blue => ",color=blue".into(),
                Color::Yellow => ",color=yellow".into(),
                Color::Purple => ",color=purple".into(),
                Color::Color(color) => format!(",color=\\\"#{color:X}\\\""),
            };
            let style = match edge.style {
                EdgeStyle::Line => "",
                EdgeStyle::Dotted => ",style=dotted",
                EdgeStyle::Dashed => ",style=dashed",
            };
            result += &format!(
                "\t\t{}_i_{} -> {}_i_{} [label=\"{}\", len=f{style}{color}];\n",
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
            let color = match node.color {
                Color::None => String::new(),
                Color::Red => "red".into(),
                Color::Green => "green".into(),
                Color::Blue => "blue".into(),
                Color::Yellow => "yellow".into(),
                Color::Purple => "purple".into(),
                Color::Color(color) => format!("#{color:X}"),
            };
            let (shape_begin, shape_end) = match node.shape {
                NodeShape::None | NodeShape::Record => ('[', ']'),
                NodeShape::Diamond => ('{', '}'),
            };
            result += &format!(
                "    {}_i_{}{shape_begin}{}{shape_end}\n",
                self.label, node.location, node.label
            );
            if !color.is_empty() {
                result += &format!(
                    "    style {}_i_{} fill:{color}\n",
                    self.label, node.location
                );
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
                Color::Color(color) => format!("#{color:X}"),
            };
            let style = match edge.style {
                EdgeStyle::Line => "-->",
                EdgeStyle::Dotted => "-.->",
                EdgeStyle::Dashed => "-.->",
            };
            result += &format!(
                "    {}_i_{} {style}| {}| {}_i_{}\n",
                self.label,
                edge.from,
                edge.label.as_deref().unwrap_or(""),
                self.label,
                edge.to,
            );

            if !color.is_empty() {
                result += &format!(
                    "    linkStyle {} stroke:{}, stroke-width: 4px\n",
                    index, color
                );
            }
        }
        result += "\n";
        result
    }
}
