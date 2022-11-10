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
pub enum Direction {
    TopToBottom,
    LeftToRight,
    RightToLeft,
}

#[derive(Debug)]
pub struct SubGraph {
    label: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    direction: Direction,

    subgraphs: Vec<SubGraph>,
}

impl SubGraph {
    pub fn new(label: String) -> Self {
        Self {
            label,
            nodes: Vec::default(),
            edges: Vec::default(),
            direction: Direction::TopToBottom,
            subgraphs: Vec::default(),
        }
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
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

    pub fn subgraph(&mut self, label: String) -> &mut SubGraph {
        self.subgraphs.push(SubGraph::new(label));
        self.subgraphs
            .last_mut()
            .expect("We just pushed a subgraph")
    }

    fn graphviz_format(&self, result: &mut String) {
        result.push_str(&format!("\tsubgraph cluster_{} {{\n", self.label));
        result.push_str("\t\tstyle = filled;\n");
        result.push_str(&format!("\t\tlabel = \"{}\";\n", self.label));
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
            result.push_str(&format!(
                "\t\t{}_i_{}[label=\"{}\"{shape}{color}];\n",
                self.label, node.location, node.label
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
                Color::Color(color) => format!(",color=\\\"#{color:X}\\\""),
            };
            let style = match edge.style {
                EdgeStyle::Line => "",
                EdgeStyle::Dotted => ",style=dotted",
                EdgeStyle::Dashed => ",style=dashed",
            };
            result.push_str(&format!(
                "\t\t{}_i_{} -> {}_i_{} [label=\"{}\", len=f{style}{color}];\n",
                self.label,
                edge.from,
                self.label,
                edge.to,
                edge.label.as_deref().unwrap_or("")
            ));
        }
        for subgraph in &self.subgraphs {
            subgraph.graphviz_format(result);
        }
        result.push_str("\t}\n");
    }

    fn mermaid_format(&self, result: &mut String) {
        let rankdir = match self.direction {
            Direction::TopToBottom => "TB",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result.push_str(&format!("  subgraph {}\n", self.label));
        result.push_str(&format!("  direction {}\n", rankdir));
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
            result.push_str(&format!(
                "  {}_i_{}{shape_begin}\"{}\"{shape_end}\n",
                self.label, node.location, node.label
            ));
            if !color.is_empty() {
                result.push_str(&format!(
                    "  style {}_i_{} fill:{color}\n",
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
                Color::Color(color) => format!("#{color:X}"),
            };
            let style = match edge.style {
                EdgeStyle::Line => "-->",
                EdgeStyle::Dotted => "-.->",
                EdgeStyle::Dashed => "-.->",
            };
            result.push_str(&format!(
                "  {}_i_{} {style}| {}| {}_i_{}\n",
                self.label,
                edge.from,
                edge.label.as_deref().unwrap_or(""),
                self.label,
                edge.to,
            ));

            if !color.is_empty() {
                result.push_str(&format!(
                    "  linkStyle {} stroke:{}, stroke-width: 4px\n",
                    index, color
                ));
            }
        }
        for subgraph in &self.subgraphs {
            subgraph.mermaid_format(result);
        }
        result.push_str("  end\n");
    }
}

#[derive(Debug)]
pub struct Graph {
    subgraphs: Vec<SubGraph>,
    direction: Direction,
}

impl Graph {
    pub fn new(direction: Direction) -> Self {
        Graph {
            subgraphs: Vec::default(),
            direction,
        }
    }

    pub fn subgraph(&mut self, label: String) -> &mut SubGraph {
        self.subgraphs.push(SubGraph::new(label));
        let result = self
            .subgraphs
            .last_mut()
            .expect("We just pushed a subgraph");
        result.set_direction(self.direction);
        result
    }

    pub fn to_graphviz_format(&self) -> String {
        let mut result = String::new();
        result += "digraph {\n";
        result += "\tnode [shape=record];\n";

        let rankdir = match self.direction {
            Direction::TopToBottom => "TB",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result += &format!("\trankdir={rankdir};\n");

        for subgraph in &self.subgraphs {
            subgraph.graphviz_format(&mut result);
        }
        result += "}\n";
        result
    }

    pub fn to_mermaid_format(&self) -> String {
        let mut result = String::new();
        let rankdir = match self.direction {
            Direction::TopToBottom => "TD",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result += &format!("graph {}\n", rankdir);

        for subgraph in &self.subgraphs {
            subgraph.mermaid_format(&mut result);
        }
        result += "\n";
        result
    }
}
