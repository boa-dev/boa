use std::mem::size_of;

use boa_interner::{Interner, Sym};

use crate::vm::{CodeBlock, Opcode};

#[allow(clippy::many_single_char_names)]
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> u32 {
    let h_i = (h * 6.0) as i64;
    let f = h * 6.0 - h_i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match h_i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => unreachable!(),
    };

    let r = (r * 256.0) as u32;
    let g = (g * 256.0) as u32;
    let b = (b * 256.0) as u32;

    let mut result = 0;
    result |= r << 16;
    result |= g << 8;
    result |= b;

    result
}

fn generate_color(mut random: f64) -> u32 {
    const GOLDEN_RATIO_CONJUGATE: f64 = 0.618033988749895;
    random += GOLDEN_RATIO_CONJUGATE;
    random %= 1.0;
    hsv_to_rgb(random, 0.7, 0.95)
}

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

#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    None,
    Arrow,
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
    from: usize,
    to: usize,
    label: Option<Box<str>>,
    color: Color,
    style: EdgeStyle,
    type_: EdgeType,
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
            type_: EdgeType::Arrow,
        }
    }

    pub fn set_type(&mut self, type_: EdgeType) {
        self.type_ = type_;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    TopToBottom,
    BottomToTop,
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
    ) -> &mut Edge {
        let edge = Edge::new(from, to, label, color, style);
        self.edges.push(edge);
        self.edges.last_mut().expect("Already pushed edge")
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

        result.push_str(&format!(
            "\t\t{}_start [label=\"Start\",shape=Mdiamond,style=filled,color=green]\n",
            self.label
        ));
        if !self.nodes.is_empty() {
            result.push_str(&format!("\t\t{}_start -> {}_i_0\n", self.label, self.label));
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
                Color::Color(color) => format!(",style=filled,color=\"#{color:X}\""),
            };
            result.push_str(&format!(
                "\t\t{}_i_{}[label=\"{:04}: {}\"{shape}{color}];\n",
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
                Color::Color(color) => format!(",color=\"#{color:X}\""),
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
            Direction::BottomToTop => "BT",
            Direction::LeftToRight => "LR",
            Direction::RightToLeft => "RL",
        };
        result.push_str(&format!("  subgraph {}\n", self.label));
        result.push_str(&format!("  direction {}\n", rankdir));

        result.push_str(&format!("  {}_start{{Start}}\n", self.label));
        result.push_str(&format!("  style {}_start fill:green\n", self.label));
        if !self.nodes.is_empty() {
            result.push_str(&format!("  {}_start --> {}_i_0\n", self.label, self.label));
        }

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
                "  {}_i_{}{shape_begin}\"{:04}: {}\"{shape_end}\n",
                self.label, node.location, node.location, node.label
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
            let style = match (edge.style, edge.type_) {
                (EdgeStyle::Line, EdgeType::None) => "---",
                (EdgeStyle::Line, EdgeType::Arrow) => "-->",
                (EdgeStyle::Dotted | EdgeStyle::Dashed, EdgeType::None) => "-.-",
                (EdgeStyle::Dotted | EdgeStyle::Dashed, EdgeType::Arrow) => "-.->",
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
                    index + 1,
                    color
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
            Direction::BottomToTop => "BT",
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
            Direction::BottomToTop => "DT",
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

impl CodeBlock {
    pub fn to_graph(&self, interner: &Interner, graph: &mut SubGraph) {
        let mut name = interner.resolve_expect(self.name).to_string();
        if self.name == Sym::MAIN {
            name = "__main__".to_string();
        }

        graph.set_label(name);

        let mut environments = Vec::new();
        let mut try_entries = Vec::new();
        let mut returns = Vec::new();

        let mut pc = 0;
        while pc < self.code.len() {
            let opcode: Opcode = self.code[pc].try_into().expect("invalid opcode");
            let opcode_str = opcode.as_str();
            let previous_pc = pc;

            let mut tmp = pc;
            let label = format!(
                "{opcode_str} {}",
                self.instruction_operands(&mut tmp, interner)
            );

            pc += size_of::<Opcode>();
            match opcode {
                Opcode::RotateLeft | Opcode::RotateRight => {
                    pc += size_of::<u8>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushInt8 => {
                    pc += size_of::<i8>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushInt16 => {
                    pc += size_of::<i16>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushInt32 => {
                    pc += size_of::<i32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushRational => {
                    pc += size_of::<f64>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushLiteral => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let operand_str = self.literals[operand as usize].display().to_string();
                    let operand_str = operand_str.escape_debug();
                    let label = format!("{opcode_str} {}", operand_str);

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Jump => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        operand as usize,
                        None,
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Opcode::JumpIfFalse
                | Opcode::JumpIfNotUndefined
                | Opcode::JumpIfNullOrUndefined => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        operand as usize,
                        Some("YES".into()),
                        Color::Green,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        pc,
                        Some("NO".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                }
                Opcode::LogicalAnd | Opcode::LogicalOr | Opcode::Coalesce => {
                    let exit = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        exit as usize,
                        Some("SHORT CIRCUIT".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                }
                Opcode::Case => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        pc,
                        Some("NO".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        address,
                        Some("YES".into()),
                        Color::Green,
                        EdgeStyle::Line,
                    );
                }
                Opcode::Default => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, address, None, Color::None, EdgeStyle::Line);
                }
                Opcode::ForInLoopInitIterator => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        address,
                        Some("NULL OR UNDEFINED".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Opcode::ForInLoopNext
                | Opcode::ForAwaitOfLoopNext
                | Opcode::GeneratorNextDelegate => {
                    let address = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        address,
                        Some("DONE".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Opcode::CatchStart
                | Opcode::FinallySetJump
                | Opcode::CallEval
                | Opcode::Call
                | Opcode::New
                | Opcode::SuperCall
                | Opcode::ConcatToString => {
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::TryStart => {
                    let next_address = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let finally_address = self.read::<u32>(pc);
                    pc += size_of::<u32>();

                    try_entries.push((
                        previous_pc,
                        next_address,
                        if finally_address == 0 {
                            None
                        } else {
                            Some(finally_address)
                        },
                    ));

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        next_address as usize,
                        Some("NEXT".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );
                    if finally_address != 0 {
                        graph.add_edge(
                            previous_pc,
                            finally_address as usize,
                            Some("FINALLY".into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Opcode::CopyDataProperties => {
                    let operand1 = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let operand2 = self.read::<u32>(pc);
                    pc += size_of::<u32>();

                    let label = format!("{opcode_str} {operand1}, {operand2}");
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushDeclarativeEnvironment | Opcode::PushFunctionEnvironment => {
                    let random = rand::random();
                    environments.push((previous_pc, random));

                    pc += size_of::<u32>();
                    pc += size_of::<u32>();

                    graph.add_node(
                        previous_pc,
                        NodeShape::None,
                        label.into(),
                        Color::Color(generate_color(random)),
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PopEnvironment => {
                    let (environment_push, random) = environments
                        .pop()
                        .expect("There should be a push evironment before");

                    let color = generate_color(random);
                    graph.add_node(
                        previous_pc,
                        NodeShape::None,
                        label.into(),
                        Color::Color(color),
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph
                        .add_edge(
                            previous_pc,
                            environment_push,
                            None,
                            Color::Color(color),
                            EdgeStyle::Dotted,
                        )
                        .set_type(EdgeType::None);
                }
                Opcode::GetArrowFunction
                | Opcode::GetAsyncArrowFunction
                | Opcode::GetFunction
                | Opcode::GetFunctionAsync
                | Opcode::GetGenerator
                | Opcode::GetGeneratorAsync => {
                    let operand = self.read::<u32>(pc);
                    let fn_name = interner
                        .resolve_expect(self.functions[operand as usize].name)
                        .to_string();
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{fn_name}' (length: {})",
                        self.functions[operand as usize].length
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::DefInitArg
                | Opcode::DefVar
                | Opcode::DefInitVar
                | Opcode::DefLet
                | Opcode::DefInitLet
                | Opcode::DefInitConst
                | Opcode::GetName
                | Opcode::GetNameOrUndefined
                | Opcode::SetName
                | Opcode::DeleteName => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{}'",
                        interner.resolve_expect(self.bindings[operand as usize].name().sym()),
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::GetPropertyByName
                | Opcode::SetPropertyByName
                | Opcode::DefineOwnPropertyByName
                | Opcode::DefineClassMethodByName
                | Opcode::SetPropertyGetterByName
                | Opcode::DefineClassGetterByName
                | Opcode::SetPropertySetterByName
                | Opcode::DefineClassSetterByName
                | Opcode::AssignPrivateField
                | Opcode::SetPrivateField
                | Opcode::SetPrivateMethod
                | Opcode::SetPrivateSetter
                | Opcode::SetPrivateGetter
                | Opcode::GetPrivateField
                | Opcode::DeletePropertyByName
                | Opcode::PushClassFieldPrivate
                | Opcode::PushClassPrivateGetter
                | Opcode::PushClassPrivateSetter
                | Opcode::PushClassPrivateMethod => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{}'",
                        interner.resolve_expect(self.names[operand as usize].sym()),
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Throw => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    if let Some((_try_pc, next, _finally)) = try_entries.last() {
                        graph.add_edge(
                            previous_pc,
                            *next as usize,
                            Some("CAUGHT".into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Opcode::Pop
                | Opcode::PopIfThrown
                | Opcode::Dup
                | Opcode::Swap
                | Opcode::PushZero
                | Opcode::PushOne
                | Opcode::PushNaN
                | Opcode::PushPositiveInfinity
                | Opcode::PushNegativeInfinity
                | Opcode::PushNull
                | Opcode::PushTrue
                | Opcode::PushFalse
                | Opcode::PushUndefined
                | Opcode::PushEmptyObject
                | Opcode::PushClassPrototype
                | Opcode::SetClassPrototype
                | Opcode::SetHomeObject
                | Opcode::Add
                | Opcode::Sub
                | Opcode::Div
                | Opcode::Mul
                | Opcode::Mod
                | Opcode::Pow
                | Opcode::ShiftRight
                | Opcode::ShiftLeft
                | Opcode::UnsignedShiftRight
                | Opcode::BitOr
                | Opcode::BitAnd
                | Opcode::BitXor
                | Opcode::BitNot
                | Opcode::In
                | Opcode::Eq
                | Opcode::StrictEq
                | Opcode::NotEq
                | Opcode::StrictNotEq
                | Opcode::GreaterThan
                | Opcode::GreaterThanOrEq
                | Opcode::LessThan
                | Opcode::LessThanOrEq
                | Opcode::InstanceOf
                | Opcode::TypeOf
                | Opcode::Void
                | Opcode::LogicalNot
                | Opcode::Pos
                | Opcode::Neg
                | Opcode::Inc
                | Opcode::IncPost
                | Opcode::Dec
                | Opcode::DecPost
                | Opcode::GetPropertyByValue
                | Opcode::GetPropertyByValuePush
                | Opcode::SetPropertyByValue
                | Opcode::DefineOwnPropertyByValue
                | Opcode::DefineClassMethodByValue
                | Opcode::SetPropertyGetterByValue
                | Opcode::DefineClassGetterByValue
                | Opcode::SetPropertySetterByValue
                | Opcode::DefineClassSetterByValue
                | Opcode::DeletePropertyByValue
                | Opcode::DeleteSuperThrow
                | Opcode::ToPropertyKey
                | Opcode::ToBoolean
                | Opcode::CatchEnd
                | Opcode::CatchEnd2
                | Opcode::FinallyStart
                | Opcode::FinallyEnd
                | Opcode::This
                | Opcode::Super
                | Opcode::LoopStart
                | Opcode::LoopContinue
                | Opcode::LoopEnd
                | Opcode::InitIterator
                | Opcode::InitIteratorAsync
                | Opcode::IteratorNext
                | Opcode::IteratorClose
                | Opcode::IteratorToArray
                | Opcode::RequireObjectCoercible
                | Opcode::ValueNotNullOrUndefined
                | Opcode::RestParameterInit
                | Opcode::RestParameterPop
                | Opcode::PushValueToArray
                | Opcode::PushElisionToArray
                | Opcode::PushIteratorToArray
                | Opcode::PushNewArray
                | Opcode::PopOnReturnAdd
                | Opcode::PopOnReturnSub
                | Opcode::Yield
                | Opcode::GeneratorNext
                | Opcode::AsyncGeneratorNext
                | Opcode::PushClassField
                | Opcode::SuperCallDerived
                | Opcode::Await
                | Opcode::PushNewTarget
                | Opcode::CallEvalSpread
                | Opcode::CallSpread
                | Opcode::NewSpread
                | Opcode::SuperCallSpread
                | Opcode::ForAwaitOfLoopIterate
                | Opcode::SetPrototype
                | Opcode::Nop => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::TryEnd => {
                    try_entries
                        .pop()
                        .expect("there should already be try block");

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Return => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    if let Some((_try_pc, _next, Some(finally))) = try_entries.last() {
                        graph.add_edge(
                            previous_pc,
                            *finally as usize,
                            None,
                            Color::None,
                            EdgeStyle::Line,
                        );
                    } else {
                        returns.push(previous_pc);
                    }
                }
            }
        }

        for ret in returns {
            graph.add_edge(ret, pc, None, Color::None, EdgeStyle::Line);
        }

        graph.add_node(pc, NodeShape::Diamond, "End".into(), Color::Red);

        for function in &self.functions {
            let subgraph = graph.subgraph(String::new());
            function.to_graph(interner, subgraph);
        }
    }
}
