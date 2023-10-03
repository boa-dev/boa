//! This module is responsible for generating the vm instruction flowgraph.

use crate::vm::CodeBlock;
use boa_interner::Interner;
use boa_macros::utf16;

mod color;
mod edge;
mod graph;
mod node;

pub use color::*;
pub use edge::*;
pub use graph::*;
pub use node::*;

use super::{Instruction, InstructionIterator};

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

        let mut iterator = InstructionIterator::new(&self.bytecode);
        while let Some((previous_pc, _, instruction)) = iterator.next() {
            let opcode = instruction.opcode();
            let opcode_str = opcode.as_str();

            let label = format!(
                "{opcode_str} {}",
                self.instruction_operands(&instruction, interner)
            );

            let pc = iterator.pc();

            match instruction {
                Instruction::SetFunctionName { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::RotateLeft { .. }
                | Instruction::RotateRight { .. }
                | Instruction::CreateIteratorResult { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::Generator { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushInt8 { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushInt16 { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushInt32 { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushFloat { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushDouble { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushLiteral { .. } | Instruction::PushRegExp { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::Jump { address } => {
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        address as usize,
                        None,
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Instruction::JumpIfFalse { address }
                | Instruction::JumpIfTrue { address }
                | Instruction::JumpIfNotUndefined { address }
                | Instruction::JumpIfNullOrUndefined { address } => {
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        address as usize,
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
                Instruction::TemplateLookup { .. } | Instruction::TemplateCreate { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::Red);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::LogicalAnd { exit }
                | Instruction::LogicalOr { exit }
                | Instruction::Coalesce { exit } => {
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
                Instruction::Case { address } => {
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
                        address as usize,
                        Some("YES".into()),
                        Color::Green,
                        EdgeStyle::Line,
                    );
                }
                Instruction::Default { address } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        address as usize,
                        None,
                        Color::None,
                        EdgeStyle::Line,
                    );
                }
                Instruction::GeneratorDelegateNext {
                    return_method_undefined,
                    throw_method_undefined,
                } => {
                    graph.add_node(
                        previous_pc,
                        NodeShape::Diamond,
                        opcode_str.into(),
                        Color::None,
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        throw_method_undefined as usize,
                        Some("`throw` undefined".into()),
                        Color::Red,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        return_method_undefined as usize,
                        Some("`return` undefined".into()),
                        Color::Blue,
                        EdgeStyle::Line,
                    );
                }
                Instruction::GeneratorDelegateResume { r#return, exit } => {
                    graph.add_node(
                        previous_pc,
                        NodeShape::Diamond,
                        opcode_str.into(),
                        Color::None,
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                    graph.add_edge(
                        previous_pc,
                        r#return as usize,
                        Some("return".into()),
                        Color::Yellow,
                        EdgeStyle::Line,
                    );
                    graph.add_edge(
                        previous_pc,
                        exit as usize,
                        Some("done".into()),
                        Color::Blue,
                        EdgeStyle::Line,
                    );
                }
                Instruction::CallEval { .. }
                | Instruction::Call { .. }
                | Instruction::New { .. }
                | Instruction::SuperCall { .. }
                | Instruction::ConcatToString { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::JumpIfNotResumeKind { exit, .. } => {
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
                Instruction::CopyDataProperties { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PushDeclarativeEnvironment { .. } => {
                    let random = rand::random();

                    graph.add_node(
                        previous_pc,
                        NodeShape::None,
                        label.into(),
                        Color::from_random_number(random),
                    );
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::PopEnvironment => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::GetArrowFunction { .. }
                | Instruction::GetAsyncArrowFunction { .. }
                | Instruction::GetFunction { .. }
                | Instruction::GetFunctionAsync { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::GetGenerator { .. } | Instruction::GetGeneratorAsync { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::DefVar { .. }
                | Instruction::DefInitVar { .. }
                | Instruction::PutLexicalValue { .. }
                | Instruction::GetName { .. }
                | Instruction::GetLocator { .. }
                | Instruction::GetNameAndLocator { .. }
                | Instruction::GetNameOrUndefined { .. }
                | Instruction::SetName { .. }
                | Instruction::DeleteName { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::GetPropertyByName { .. }
                | Instruction::SetPropertyByName { .. }
                | Instruction::DefineOwnPropertyByName { .. }
                | Instruction::DefineClassStaticMethodByName { .. }
                | Instruction::DefineClassMethodByName { .. }
                | Instruction::SetPropertyGetterByName { .. }
                | Instruction::DefineClassStaticGetterByName { .. }
                | Instruction::DefineClassGetterByName { .. }
                | Instruction::SetPropertySetterByName { .. }
                | Instruction::DefineClassStaticSetterByName { .. }
                | Instruction::DefineClassSetterByName { .. }
                | Instruction::SetPrivateField { .. }
                | Instruction::DefinePrivateField { .. }
                | Instruction::SetPrivateMethod { .. }
                | Instruction::SetPrivateSetter { .. }
                | Instruction::SetPrivateGetter { .. }
                | Instruction::GetPrivateField { .. }
                | Instruction::DeletePropertyByName { .. }
                | Instruction::PushClassFieldPrivate { .. }
                | Instruction::PushClassPrivateGetter { .. }
                | Instruction::PushClassPrivateSetter { .. }
                | Instruction::PushClassPrivateMethod { .. }
                | Instruction::InPrivate { .. }
                | Instruction::ThrowMutateImmutable { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::ThrowNewTypeError { .. } => {
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
                Instruction::Throw | Instruction::ReThrow => {
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
                Instruction::PushPrivateEnvironment { .. } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::JumpTable { default, addresses } => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(
                        previous_pc,
                        default as usize,
                        Some("DEFAULT".into()),
                        Color::None,
                        EdgeStyle::Line,
                    );

                    for (i, address) in addresses.iter().enumerate() {
                        graph.add_edge(
                            previous_pc,
                            *address as usize,
                            Some(format!("Index: {i}").into()),
                            Color::None,
                            EdgeStyle::Line,
                        );
                    }
                }
                Instruction::Pop
                | Instruction::Dup
                | Instruction::Swap
                | Instruction::PushZero
                | Instruction::PushOne
                | Instruction::PushNaN
                | Instruction::PushPositiveInfinity
                | Instruction::PushNegativeInfinity
                | Instruction::PushNull
                | Instruction::PushTrue
                | Instruction::PushFalse
                | Instruction::PushUndefined
                | Instruction::PushEmptyObject
                | Instruction::PushClassPrototype
                | Instruction::SetClassPrototype
                | Instruction::SetHomeObject
                | Instruction::Add
                | Instruction::Sub
                | Instruction::Div
                | Instruction::Mul
                | Instruction::Mod
                | Instruction::Pow
                | Instruction::ShiftRight
                | Instruction::ShiftLeft
                | Instruction::UnsignedShiftRight
                | Instruction::BitOr
                | Instruction::BitAnd
                | Instruction::BitXor
                | Instruction::BitNot
                | Instruction::In
                | Instruction::Eq
                | Instruction::StrictEq
                | Instruction::NotEq
                | Instruction::StrictNotEq
                | Instruction::GreaterThan
                | Instruction::GreaterThanOrEq
                | Instruction::LessThan
                | Instruction::LessThanOrEq
                | Instruction::InstanceOf
                | Instruction::TypeOf
                | Instruction::Void
                | Instruction::LogicalNot
                | Instruction::Pos
                | Instruction::Neg
                | Instruction::Inc
                | Instruction::IncPost
                | Instruction::Dec
                | Instruction::DecPost
                | Instruction::GetPropertyByValue
                | Instruction::GetPropertyByValuePush
                | Instruction::SetPropertyByValue
                | Instruction::DefineOwnPropertyByValue
                | Instruction::DefineClassStaticMethodByValue
                | Instruction::DefineClassMethodByValue
                | Instruction::SetPropertyGetterByValue
                | Instruction::DefineClassStaticGetterByValue
                | Instruction::DefineClassGetterByValue
                | Instruction::SetPropertySetterByValue
                | Instruction::DefineClassStaticSetterByValue
                | Instruction::DefineClassSetterByValue
                | Instruction::DeletePropertyByValue
                | Instruction::DeleteSuperThrow
                | Instruction::ToPropertyKey
                | Instruction::ToBoolean
                | Instruction::This
                | Instruction::Super
                | Instruction::IncrementLoopIteration
                | Instruction::CreateForInIterator
                | Instruction::GetIterator
                | Instruction::GetAsyncIterator
                | Instruction::IteratorNext
                | Instruction::IteratorNextWithoutPop
                | Instruction::IteratorFinishAsyncNext
                | Instruction::IteratorValue
                | Instruction::IteratorValueWithoutPop
                | Instruction::IteratorResult
                | Instruction::IteratorDone
                | Instruction::IteratorToArray
                | Instruction::IteratorPop
                | Instruction::IteratorReturn
                | Instruction::IteratorStackEmpty
                | Instruction::RequireObjectCoercible
                | Instruction::ValueNotNullOrUndefined
                | Instruction::RestParameterInit
                | Instruction::RestParameterPop
                | Instruction::PushValueToArray
                | Instruction::PushElisionToArray
                | Instruction::PushIteratorToArray
                | Instruction::PushNewArray
                | Instruction::GeneratorYield
                | Instruction::AsyncGeneratorYield
                | Instruction::AsyncGeneratorClose
                | Instruction::CreatePromiseCapability
                | Instruction::CompletePromiseCapability
                | Instruction::GeneratorNext
                | Instruction::PushClassField
                | Instruction::SuperCallDerived
                | Instruction::Await
                | Instruction::NewTarget
                | Instruction::ImportMeta
                | Instruction::CallEvalSpread
                | Instruction::CallSpread
                | Instruction::NewSpread
                | Instruction::SuperCallSpread
                | Instruction::SuperCallPrepare
                | Instruction::SetPrototype
                | Instruction::IsObject
                | Instruction::SetNameByLocator
                | Instruction::PushObjectEnvironment
                | Instruction::PopPrivateEnvironment
                | Instruction::ImportCall
                | Instruction::GetReturnValue
                | Instruction::SetReturnValue
                | Instruction::Exception
                | Instruction::MaybeException
                | Instruction::Nop => {
                    graph.add_node(previous_pc, NodeShape::None, label.into(), Color::None);
                    graph.add_edge(previous_pc, pc, None, Color::None, EdgeStyle::Line);
                }
                Instruction::Return => {
                    graph.add_node(previous_pc, NodeShape::Diamond, label.into(), Color::Red);
                }
                Instruction::U16Operands
                | Instruction::U32Operands
                | Instruction::Reserved1
                | Instruction::Reserved2
                | Instruction::Reserved3
                | Instruction::Reserved4
                | Instruction::Reserved5
                | Instruction::Reserved6
                | Instruction::Reserved7
                | Instruction::Reserved8
                | Instruction::Reserved9
                | Instruction::Reserved10
                | Instruction::Reserved11
                | Instruction::Reserved12
                | Instruction::Reserved13
                | Instruction::Reserved14
                | Instruction::Reserved15
                | Instruction::Reserved16
                | Instruction::Reserved17
                | Instruction::Reserved18
                | Instruction::Reserved19
                | Instruction::Reserved20
                | Instruction::Reserved21
                | Instruction::Reserved22
                | Instruction::Reserved23
                | Instruction::Reserved24
                | Instruction::Reserved25
                | Instruction::Reserved26
                | Instruction::Reserved27
                | Instruction::Reserved28
                | Instruction::Reserved29
                | Instruction::Reserved30
                | Instruction::Reserved31
                | Instruction::Reserved32
                | Instruction::Reserved33
                | Instruction::Reserved34
                | Instruction::Reserved35
                | Instruction::Reserved36
                | Instruction::Reserved37
                | Instruction::Reserved38
                | Instruction::Reserved39
                | Instruction::Reserved40
                | Instruction::Reserved41
                | Instruction::Reserved42
                | Instruction::Reserved43
                | Instruction::Reserved44
                | Instruction::Reserved45
                | Instruction::Reserved46
                | Instruction::Reserved47
                | Instruction::Reserved48
                | Instruction::Reserved49
                | Instruction::Reserved50
                | Instruction::Reserved51
                | Instruction::Reserved52
                | Instruction::Reserved53
                | Instruction::Reserved54
                | Instruction::Reserved55
                | Instruction::Reserved56
                | Instruction::Reserved57
                | Instruction::Reserved58 => unreachable!("Reserved opcodes are unrechable"),
            }
        }

        for function in self.functions.as_ref() {
            let subgraph = graph.subgraph(String::new());
            function.to_graph(interner, subgraph);
        }
    }
}
