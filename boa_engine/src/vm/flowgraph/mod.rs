//! This module is responsible for generating the vm instruction flowgraph.

use crate::vm::{CodeBlock, Opcode};
use boa_interner::Interner;
use boa_macros::utf16;
use std::mem::size_of;

mod color;
mod edge;
mod graph;
mod node;

pub use color::*;
pub use edge::*;
pub use graph::*;
pub use node::*;

impl CodeBlock {
    /// Output the [`CodeBlock`] VM instructions into a [`Graph`].
    #[allow(clippy::match_same_arms)]
    pub fn to_graph(&self, interner: &Interner, graph: &mut SubGraph) {
        // Have to remove any invalid graph chars like `<` or `>`.
        let name = if self.name() == utf16!("<main>") {
            "__main__".to_string()
        } else {
            self.name().to_std_string_escaped()
        };

        graph.set_label(name);

        let mut pc = 0;
        while pc < self.bytecode.len() {
            let opcode: Opcode = self.bytecode[pc].into();
            let opcode_str = opcode.as_str();
            let previous_pc = pc;

            let mut tmp = pc;
            let label = format!(
                "{opcode_str} {}",
                self.instruction_operands(&mut tmp, interner)
            );

            pc += size_of::<Opcode>();
            match opcode {
                Opcode::SetFunctionName => {
                    let operand = self.read::<u8>(pc);
                    pc += size_of::<u8>();
                    let label = format!(
                        "{opcode_str} {}",
                        match operand {
                            0 => "prefix: none",
                            1 => "prefix: get",
                            2 => "prefix: set",
                            _ => unreachable!(),
                        }
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::RotateLeft | Opcode::RotateRight | Opcode::CreateIteratorResult => {
                    pc += size_of::<u8>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Generator => {
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
                Opcode::PushFloat => {
                    pc += size_of::<f32>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushDouble => {
                    pc += size_of::<f64>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PushLiteral => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let operand_str = self.literals[operand as usize].display().to_string();
                    let operand_str = operand_str.escape_debug();
                    let label = format!("{opcode_str} {operand_str}");

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
                | Opcode::JumpIfTrue
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
                Opcode::TemplateLookup | Opcode::TemplateCreate => {
                    let start_address = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let end_address = self.read::<u64>(pc);
                    pc += size_of::<u64>();

                    let label = format!("{opcode_str} {start_address}, {end_address}");
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::Red);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
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
                Opcode::GeneratorDelegateNext => {
                    let throw_method_undefined = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    let return_method_undefined = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(
                        previous_pc,
                        NodeShape::Diamond,
                        opcode_str.into(),
                        Color::None,
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        throw_method_undefined,
                        Some("`throw` undefined".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        return_method_undefined,
                        Some("`return` undefined".into()),
                        Color::Blue,
                        EdgeStyle::Line,
                    );
                }
                Opcode::GeneratorDelegateResume => {
                    let return_gen = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    let exit = self.read::<u32>(pc) as usize;
                    pc += size_of::<u32>();
                    graph.add_node(
                        previous_pc,
                        NodeShape::Diamond,
                        opcode_str.into(),
                        Color::None,
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        return_gen,
                        Some("return".into()),
                        Color::Yellow,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        exit,
                        Some("done".into()),
                        Color::Blue,
                        EdgeStyle::Line,
                    );
                }
                Opcode::CallEval
                | Opcode::Call
                | Opcode::New
                | Opcode::SuperCall
                | Opcode::ConcatToString => {
                    pc += size_of::<u32>();
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::JumpIfNotResumeKind => {
                    let exit = self.read::<u32>(pc);
                    pc += size_of::<u32>();

                    let _resume_kind = self.read::<u8>(pc);
                    pc += size_of::<u8>();

                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        exit as usize,
                        Some("EXIT".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(previous_pc, pc, None, Color::Green, EdgeStyle::Line);
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

                    pc += size_of::<u32>();

                    graph.add_node(
                        previous_pc,
                        NodeShape::None,
                        label.into(),
                        Color::from_random_number(random),
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::PopEnvironment => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::GetArrowFunction
                | Opcode::GetAsyncArrowFunction
                | Opcode::GetFunction
                | Opcode::GetFunctionAsync => {
                    let operand = self.read::<u32>(pc);
                    let fn_name = self.functions[operand as usize]
                        .name()
                        .to_std_string_escaped();
                    pc += size_of::<u32>() + size_of::<u8>();
                    let label = format!(
                        "{opcode_str} '{fn_name}' (length: {})",
                        self.functions[operand as usize].length
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::GetGenerator | Opcode::GetGeneratorAsync => {
                    let operand = self.read::<u32>(pc);
                    let fn_name = self.functions[operand as usize]
                        .name()
                        .to_std_string_escaped();
                    let label = format!(
                        "{opcode_str} '{fn_name}' (length: {})",
                        self.functions[operand as usize].length
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::DefVar
                | Opcode::DefInitVar
                | Opcode::PutLexicalValue
                | Opcode::GetName
                | Opcode::GetLocator
                | Opcode::GetNameAndLocator
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
                | Opcode::GetMethod
                | Opcode::SetPropertyByName
                | Opcode::DefineOwnPropertyByName
                | Opcode::DefineClassStaticMethodByName
                | Opcode::DefineClassMethodByName
                | Opcode::SetPropertyGetterByName
                | Opcode::DefineClassStaticGetterByName
                | Opcode::DefineClassGetterByName
                | Opcode::SetPropertySetterByName
                | Opcode::DefineClassStaticSetterByName
                | Opcode::DefineClassSetterByName
                | Opcode::SetPrivateField
                | Opcode::DefinePrivateField
                | Opcode::SetPrivateMethod
                | Opcode::SetPrivateSetter
                | Opcode::SetPrivateGetter
                | Opcode::GetPrivateField
                | Opcode::DeletePropertyByName
                | Opcode::PushClassFieldPrivate
                | Opcode::PushClassPrivateGetter
                | Opcode::PushClassPrivateSetter
                | Opcode::PushClassPrivateMethod
                | Opcode::InPrivate
                | Opcode::ThrowMutateImmutable => {
                    let operand = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let label = format!(
                        "{opcode_str} '{}'",
                        self.names[operand as usize].to_std_string_escaped(),
                    );
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::ThrowNewTypeError => {
                    pc += size_of::<u32>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    if let Some((i, handler)) = self.find_handler(previous_pc as u32) {
                        graph.add_edge(
                            previous_pc,
                            handler.handler() as usize,
                            Some(format!("Handler {i:2}: CAUGHT").into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Opcode::Throw | Opcode::ReThrow => {
                    if let Some((i, handler)) = self.find_handler(previous_pc as u32) {
                        graph.add_node(previous_pc, NodeShape::Record, label.into(), Color::None);
                        graph.add_edge(
                            previous_pc,
                            handler.handler() as usize,
                            Some(format!("Handler {i:2}: CAUGHT").into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    } else {
                        graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    }
                }
                Opcode::PushPrivateEnvironment => {
                    let count = self.read::<u32>(pc);
                    pc += size_of::<u32>() * (count as usize + 1);
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::JumpTable => {
                    let count = self.read::<u32>(pc);
                    pc += size_of::<u32>();
                    let default = self.read::<u32>(pc);
                    pc += size_of::<u32>();

                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        default as usize,
                        Some("DEFAULT".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );

                    for i in 0..count {
                        let address = self.read::<u32>(pc);
                        pc += size_of::<u32>();

                        graph.add_edge(
                            previous_pc,
                            address as usize,
                            Some(format!("Index: {i}").into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Opcode::Pop
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
                | Opcode::SetHomeObjectClass
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
                | Opcode::DefineClassStaticMethodByValue
                | Opcode::DefineClassMethodByValue
                | Opcode::SetPropertyGetterByValue
                | Opcode::DefineClassStaticGetterByValue
                | Opcode::DefineClassGetterByValue
                | Opcode::SetPropertySetterByValue
                | Opcode::DefineClassStaticSetterByValue
                | Opcode::DefineClassSetterByValue
                | Opcode::DeletePropertyByValue
                | Opcode::DeleteSuperThrow
                | Opcode::ToPropertyKey
                | Opcode::ToBoolean
                | Opcode::This
                | Opcode::Super
                | Opcode::IncrementLoopIteration
                | Opcode::CreateForInIterator
                | Opcode::GetIterator
                | Opcode::GetAsyncIterator
                | Opcode::IteratorNext
                | Opcode::IteratorNextWithoutPop
                | Opcode::IteratorFinishAsyncNext
                | Opcode::IteratorValue
                | Opcode::IteratorValueWithoutPop
                | Opcode::IteratorResult
                | Opcode::IteratorDone
                | Opcode::IteratorToArray
                | Opcode::IteratorPop
                | Opcode::IteratorReturn
                | Opcode::IteratorStackEmpty
                | Opcode::RequireObjectCoercible
                | Opcode::ValueNotNullOrUndefined
                | Opcode::RestParameterInit
                | Opcode::RestParameterPop
                | Opcode::PushValueToArray
                | Opcode::PushElisionToArray
                | Opcode::PushIteratorToArray
                | Opcode::PushNewArray
                | Opcode::GeneratorYield
                | Opcode::AsyncGeneratorYield
                | Opcode::AsyncGeneratorClose
                | Opcode::CreatePromiseCapability
                | Opcode::CompletePromiseCapability
                | Opcode::GeneratorNext
                | Opcode::PushClassField
                | Opcode::SuperCallDerived
                | Opcode::Await
                | Opcode::NewTarget
                | Opcode::ImportMeta
                | Opcode::CallEvalSpread
                | Opcode::CallSpread
                | Opcode::NewSpread
                | Opcode::SuperCallSpread
                | Opcode::SuperCallPrepare
                | Opcode::SetPrototype
                | Opcode::IsObject
                | Opcode::SetNameByLocator
                | Opcode::PushObjectEnvironment
                | Opcode::PopPrivateEnvironment
                | Opcode::ImportCall
                | Opcode::GetReturnValue
                | Opcode::SetReturnValue
                | Opcode::Exception
                | Opcode::MaybeException
                | Opcode::Nop => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Opcode::Return => {
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::Red);
                }
                Opcode::Reserved1
                | Opcode::Reserved2
                | Opcode::Reserved3
                | Opcode::Reserved4
                | Opcode::Reserved5
                | Opcode::Reserved6
                | Opcode::Reserved7
                | Opcode::Reserved8
                | Opcode::Reserved9
                | Opcode::Reserved10
                | Opcode::Reserved11
                | Opcode::Reserved12
                | Opcode::Reserved13
                | Opcode::Reserved14
                | Opcode::Reserved15
                | Opcode::Reserved16
                | Opcode::Reserved17
                | Opcode::Reserved18
                | Opcode::Reserved19
                | Opcode::Reserved20
                | Opcode::Reserved21
                | Opcode::Reserved22
                | Opcode::Reserved23
                | Opcode::Reserved24
                | Opcode::Reserved25
                | Opcode::Reserved26
                | Opcode::Reserved27
                | Opcode::Reserved28
                | Opcode::Reserved29
                | Opcode::Reserved30
                | Opcode::Reserved31
                | Opcode::Reserved32
                | Opcode::Reserved33
                | Opcode::Reserved34
                | Opcode::Reserved35
                | Opcode::Reserved36
                | Opcode::Reserved37
                | Opcode::Reserved38
                | Opcode::Reserved39
                | Opcode::Reserved40
                | Opcode::Reserved41
                | Opcode::Reserved42
                | Opcode::Reserved43
                | Opcode::Reserved44
                | Opcode::Reserved45
                | Opcode::Reserved46
                | Opcode::Reserved47
                | Opcode::Reserved48
                | Opcode::Reserved49
                | Opcode::Reserved50
                | Opcode::Reserved51
                | Opcode::Reserved52
                | Opcode::Reserved53
                | Opcode::Reserved54
                | Opcode::Reserved55
                | Opcode::Reserved56
                | Opcode::Reserved57
                | Opcode::Reserved58 => unreachable!("Reserved opcodes are unrechable"),
            }
        }

        for function in self.functions.as_ref() {
            let subgraph = graph.subgraph(String::new());
            function.to_graph(interner, subgraph);
        }
    }
}
