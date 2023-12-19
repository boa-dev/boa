//! TODO: doc

#![allow(dead_code)]
#![allow(missing_debug_implementations)]

mod basic_block;

use std::fmt::Debug;

use rustc_hash::FxHashMap;
use slotmap::{new_key_type, SlotMap};
use thin_vec::ThinVec;

use crate::vm::{CodeBlock, Handler, Instruction, InstructionIterator, Opcode};

pub use self::basic_block::BasicBlock;

new_key_type! { pub(crate) struct BasicBlockKey; }
new_key_type! { pub(crate) struct HandlerKey; }

/// TODO: doc
#[derive(Default, Clone)]
pub(crate) enum Terminator {
    /// TODO: doc
    #[default]
    None,

    /// TODO: doc
    JumpUnconditional {
        /// TODO: doc
        opcode: Opcode,
        /// TODO: doc
        target: BasicBlockKey,
    },

    /// TODO: doc
    JumpConditional {
        /// TODO: doc
        opcode: Opcode,
        /// TODO: doc
        no: BasicBlockKey,
        /// TODO: doc
        yes: BasicBlockKey,
    },

    /// TODO: doc
    TemplateLookup {
        /// TODO: doc
        no: BasicBlockKey,

        /// TODO: doc
        yes: BasicBlockKey,

        /// TODO: doc
        site: u64,
    },

    JumpTable {
        default: BasicBlockKey,
        addresses: ThinVec<BasicBlockKey>,
    },

    /// TODO: doc
    Return { end: BasicBlockKey },
}

impl Terminator {
    /// Check if [`Terminator::None`].
    #[must_use]
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Terminator::None)
    }

    /// Check if [`Terminator::Jump`].
    #[must_use]
    pub(crate) fn is_jump(&self) -> bool {
        matches!(
            self,
            Terminator::JumpUnconditional { .. } | Terminator::JumpConditional { .. }
        )
    }

    /// Check if unconditional [`Terminator::Jump`].
    #[must_use]
    pub(crate) fn is_unconditional_jump(&self) -> bool {
        matches!(self, Terminator::JumpUnconditional { .. })
    }

    /// Check if conditional [`Terminator::Jump`].
    #[must_use]
    pub(crate) fn is_conditional_jump(&self) -> bool {
        matches!(self, Terminator::JumpConditional { .. })
    }
}

/// TODO: doc
pub struct ControlFlowGraph {
    basic_block_start: BasicBlockKey,
    basic_block_end: BasicBlockKey,
    basic_blocks: SlotMap<BasicBlockKey, BasicBlock>,
    // handlers: SlotMap<HandlerKey, BasicBlockHandler>,
}

impl Debug for ControlFlowGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BasicBlocks:")?;

        let mut seen = FxHashMap::default();
        let index_from_basic_block = |bb: BasicBlockKey| {
            for (i, (key, _basic_block)) in self.basic_blocks.iter().enumerate() {
                if key == bb {
                    return i;
                }
            }

            unreachable!("There should be a basic block")
        };

        let mut index = 0;
        for key in self.basic_blocks.keys() {
            if seen.contains_key(&key) {
                continue;
            }
            seen.insert(key, index);

            let basic_block = &self.basic_blocks[key];

            write!(
                f,
                "    B{index}: -- {}reachable",
                if basic_block.reachable() { "" } else { "not " }
            )?;

            if key == self.basic_block_start {
                write!(f, " -- start")?;
            }
            if key == self.basic_block_end {
                write!(f, " -- end")?;
            }

            if !basic_block.previous.is_empty() {
                write!(f, " -- previous ")?;
                for predecessor in &basic_block.previous {
                    let index = index_from_basic_block(*predecessor);
                    write!(f, "B{index}, ")?;
                }
            }

            let next = basic_block.next();
            if !next.is_empty() {
                write!(f, " -- next ")?;
                for bb in &next {
                    let index = index_from_basic_block(*bb);
                    write!(f, "B{index}, ")?;
                }
            }

            if let Some(handler) = &basic_block.handler {
                let index = index_from_basic_block(handler.0);
                write!(f, " -- handler B{index}")?;
            }

            writeln!(f)?;

            for (i, result) in basic_block.instructions.iter().enumerate() {
                writeln!(f, "        {:06}      {}", i, result.opcode().as_str())?;
            }

            let terminator = &basic_block.terminator;
            if !terminator.is_none() {
                write!(f, "        Terminator: ")?;
                match terminator {
                    Terminator::None => write!(f, "None")?,
                    Terminator::JumpUnconditional { opcode, target } => {
                        let target = index_from_basic_block(*target);
                        write!(f, "{} B{target}", opcode.as_str())?;
                    }
                    Terminator::JumpConditional { opcode, no: _, yes } => {
                        let target = index_from_basic_block(*yes);
                        write!(f, "{} B{target}", opcode.as_str())?;
                    }
                    Terminator::TemplateLookup {
                        no: _,
                        yes,
                        site: _,
                    } => {
                        let target = index_from_basic_block(*yes);
                        write!(f, "TemplateLookup B{target}")?;
                    }
                    Terminator::JumpTable { default, addresses } => {
                        let default = index_from_basic_block(*default);
                        write!(f, "JumpTable default: B{default}")?;

                        if !addresses.is_empty() {
                            write!(f, " ")?;
                            for (i, address) in addresses.iter().enumerate() {
                                let address = index_from_basic_block(*address);
                                write!(f, "{}: B{address}", i + 1)?;
                                if i + 1 != addresses.len() {
                                    write!(f, ", ")?;
                                }
                            }
                        }
                    }
                    Terminator::Return { end } => {
                        let target = index_from_basic_block(*end);
                        write!(f, "Return B{target}")?;
                    }
                }
                writeln!(f)?;
            }

            writeln!(f)?;

            index += 1;
        }

        Ok(())
    }
}

const fn is_jump_kind_instruction(instruction: &Instruction) -> Option<u32> {
    match instruction {
        Instruction::Jump { address }
        | Instruction::JumpIfTrue { address }
        | Instruction::JumpIfFalse { address }
        | Instruction::JumpIfNotUndefined { address }
        | Instruction::JumpIfNullOrUndefined { address }
        | Instruction::Case { address }
        | Instruction::Default { address }
        | Instruction::LogicalAnd { exit: address }
        | Instruction::LogicalOr { exit: address }
        | Instruction::Coalesce { exit: address } => Some(*address),
        _ => None,
    }
}

impl ControlFlowGraph {
    /// Generate leaders for the [`BasicBlock`]s.
    fn leaders(bytecode: &[u8], handlers: &[Handler]) -> Vec<u32> {
        let mut leaders = Vec::new();

        let mut iter = InstructionIterator::new(bytecode);

        for handler in handlers {
            leaders.push(handler.start);
            leaders.push(handler.handler());
        }

        while let Some((_, _, instruction)) = iter.next() {
            // println!("{pc:4} {instruction:?}");
            match instruction {
                Instruction::Return => {
                    leaders.push(iter.pc() as u32);
                }
                Instruction::TemplateLookup { exit, .. } => {
                    leaders.push(iter.pc() as u32);
                    leaders.push(exit);
                }
                Instruction::JumpTable { default, addresses } => {
                    leaders.push(default);
                    leaders.extend_from_slice(&addresses);
                }
                instruction => {
                    if let Some(target) = is_jump_kind_instruction(&instruction) {
                        leaders.push(iter.pc() as u32);
                        leaders.push(target);
                    }
                }
            }
        }

        leaders.push(0);
        leaders.sort_unstable();
        leaders.dedup();

        // println!("leaders: {leaders:?}");

        leaders
    }

    /// TODO: doc
    #[must_use]
    pub fn generate_from_codeblock(code: &CodeBlock) -> Self {
        Self::generate(&code.bytecode, &code.handlers)
    }

    /// TODO: doc
    #[must_use]
    pub(crate) fn generate(bytecode: &[u8], handlers: &[Handler]) -> Self {
        let leaders = Self::leaders(bytecode, handlers);
        let block_count = leaders.len();

        let mut basic_block_keys = Vec::with_capacity(block_count);
        let mut basic_blocks = SlotMap::<BasicBlockKey, _>::with_capacity_and_key(block_count);
        for _ in 0..block_count {
            let key = basic_blocks.insert(BasicBlock::default());
            basic_block_keys.push(key);
        }

        let basic_block_from_bytecode_position = |address: u32| {
            let index = leaders
                .iter()
                .position(|x| *x == address)
                .expect("There should be a basic block");

            basic_block_keys[index]
        };

        let basic_block_end = basic_block_keys[leaders.len() - 1];

        let mut iter = InstructionIterator::new(bytecode);
        for (i, leader) in leaders
            .iter()
            .map(|x| *x as usize)
            .enumerate()
            .skip(1)
            .map(|(i, leader)| (i - 1, leader))
        {
            let key = basic_block_keys[i];

            let handler = handlers
                .iter()
                .rev()
                .find(|handler| handler.contains(iter.pc() as u32));
            if let Some(handler) = handler {
                let basic_block_handler = basic_block_from_bytecode_position(handler.handler());

                basic_blocks[key].handler = Some((basic_block_handler, *handler));
            }

            let mut bytecode = Vec::new();
            let mut terminator = Terminator::None;
            while let Some((_, _, instruction)) = iter.next() {
                match instruction {
                    Instruction::Return => {
                        terminator = Terminator::Return {
                            end: basic_block_end,
                        };
                    }
                    Instruction::Jump { address } | Instruction::Default { address } => {
                        let target = basic_block_from_bytecode_position(address);

                        basic_blocks[target].previous.push(key);

                        terminator = Terminator::JumpUnconditional {
                            opcode: instruction.opcode(),
                            target,
                        };
                    }
                    Instruction::TemplateLookup {
                        exit: address,
                        site,
                    } => {
                        let yes = basic_block_from_bytecode_position(address);
                        let no = basic_block_keys[i + 1];

                        basic_blocks[yes].previous.push(key);
                        basic_blocks[no].previous.push(key);

                        terminator = Terminator::TemplateLookup { no, yes, site };
                    }
                    Instruction::JumpTable { default, addresses } => {
                        let default = basic_block_from_bytecode_position(default);

                        basic_blocks[default].previous.push(key);

                        let mut basic_block_addresses = ThinVec::with_capacity(addresses.len());
                        for address in addresses {
                            let address = basic_block_from_bytecode_position(address);
                            basic_blocks[address].previous.push(key);
                            basic_block_addresses.push(address);
                        }

                        terminator = Terminator::JumpTable {
                            default,
                            addresses: basic_block_addresses,
                        }
                    }
                    instruction => {
                        if let Some(address) = is_jump_kind_instruction(&instruction) {
                            let yes = basic_block_from_bytecode_position(address);
                            let no = basic_block_keys[i + 1];

                            basic_blocks[yes].previous.push(key);
                            basic_blocks[no].previous.push(key);

                            terminator = Terminator::JumpConditional {
                                opcode: instruction.opcode(),
                                no,
                                yes,
                            };
                        } else {
                            bytecode.push(instruction);
                        }
                    }
                }

                if leader <= iter.pc() {
                    break;
                }
            }

            let basic_block = &mut basic_blocks[key];
            basic_block.instructions = bytecode;
            basic_block.terminator = terminator;
        }

        Self {
            basic_block_start: basic_block_keys[0],
            basic_block_end,
            basic_blocks,
        }
    }

    /// Remove [`BasicBlock`].
    pub(crate) fn remove(&mut self, basic_block: BasicBlockKey) {
        self.basic_blocks.remove(basic_block);
    }

    /// Finalize bytecode.
    #[must_use]
    pub fn finalize(self) -> Vec<u8> {
        let index_from_basic_block = |bb: BasicBlockKey| {
            for (i, key) in self.basic_blocks.keys().enumerate() {
                if key == bb {
                    return i;
                }
            }

            unreachable!("There should be a basic block")
        };

        let mut results = Vec::new();
        let mut labels = Vec::new();
        let mut blocks = Vec::with_capacity(self.basic_blocks.len());

        for key in self.basic_blocks.keys() {
            let basic_block = &self.basic_blocks[key];

            blocks.push(results.len() as u32);

            for instruction in &basic_block.instructions {
                instruction.to_bytecode(&mut results);
            }

            match &basic_block.terminator {
                Terminator::None => {}
                Terminator::JumpUnconditional { opcode, target } => {
                    results.extend_from_slice(&[*opcode as u8]);
                    let start = results.len();
                    results.extend_from_slice(&[0, 0, 0, 0]);

                    let target = index_from_basic_block(*target);
                    labels.push((start as u32, target));
                }
                Terminator::JumpConditional { opcode, no: _, yes } => {
                    results.extend_from_slice(&[*opcode as u8]);
                    let start = results.len();
                    results.extend_from_slice(&[0, 0, 0, 0]);

                    let target = index_from_basic_block(*yes);
                    labels.push((start as u32, target));
                }
                Terminator::TemplateLookup { yes, site, .. } => {
                    results.extend_from_slice(&[Opcode::TemplateLookup as u8]);
                    let start = results.len();
                    results.extend_from_slice(&[0, 0, 0, 0]);
                    results.extend_from_slice(&site.to_ne_bytes());

                    let target = index_from_basic_block(*yes);
                    labels.push((start as u32, target));
                }
                Terminator::JumpTable { default, addresses } => {
                    results.extend_from_slice(&[Opcode::JumpTable as u8]);
                    let start = results.len();
                    results.extend_from_slice(&[0, 0, 0, 0]);

                    let default = index_from_basic_block(*default);
                    labels.push((start as u32, default));

                    results.extend_from_slice(&(addresses.len() as u32).to_ne_bytes());

                    for address in addresses {
                        let start = results.len();
                        results.extend_from_slice(&[0, 0, 0, 0]);

                        let target = index_from_basic_block(*address);
                        labels.push((start as u32, target));
                    }
                }
                Terminator::Return { .. } => {
                    results.push(Opcode::Return as u8);
                }
            }
        }

        for (label, block_index) in labels {
            let address = blocks[block_index];

            let bytes = address.to_ne_bytes();
            results[label as usize] = bytes[0];
            results[label as usize + 1] = bytes[1];
            results[label as usize + 2] = bytes[2];
            results[label as usize + 3] = bytes[3];
        }

        results
    }
}

/// Simplifies the [`ControlFlowGraph`].
///
/// # Operations
///
/// - Conditional Branch to same blocks -> unconditional
/// - Unrachable block elimination
#[derive(Clone, Copy)]
pub struct GraphSimplification;

impl GraphSimplification {
    /// TODO: doc
    pub fn perform(_graph: &mut ControlFlowGraph) -> bool {
        // let mut changed = false;

        // for key in graph.basic_blocks.keys() {
        //     {
        //         let mut basic_block = basic_block_ptr.borrow_mut();

        //         #[allow(clippy::single_match)]
        //         match basic_block.terminator.clone() {
        //             Terminator::JumpConditional { no, yes, .. } => {
        //                 if no == yes {
        //                     basic_block.insert_last(Instruction::Pop);
        //                     basic_block.terminator = Terminator::JumpUnconditional {
        //                         opcode: Opcode::Jump,
        //                         target: yes,
        //                     };

        //                     changed |= true;
        //                 }
        //             }
        //             _ => {}
        //         }
        //     }
        // }
        // changed
        false
    }
}

/// TODO: doc
#[derive(Clone, Copy)]
pub struct GraphEliminateUnreachableBasicBlocks;

impl GraphEliminateUnreachableBasicBlocks {
    /// TODO: doc
    pub fn perform(_graph: &mut ControlFlowGraph) -> bool {
        // let mut changed = false;

        // let mut stack = vec![graph.basic_block_start.clone()];
        // while let Some(basic_block_ptr) = stack.pop() {
        //     let mut basic_block = basic_block_ptr.borrow_mut();
        //     if basic_block.reachable() {
        //         break;
        //     }
        //     basic_block.flags |= BasicBlockFlags::REACHABLE;
        //     basic_block.next(&mut stack);

        //     // println!("{:p} -- {}", basic_block_ptr.as_ptr(), basic_block.reachable());
        // }

        // assert!(
        //     graph.basic_block_start.borrow().reachable(),
        //     "start basic block node should always be reachable"
        // );

        // let mut delete_list = Vec::new();
        // for (i, basic_block) in graph.basic_blocks.iter().enumerate().rev() {
        //     if !basic_block.borrow().reachable() {
        //         delete_list.push(i);
        //     }
        // }

        // // println!("{delete_list:?}");

        // for i in delete_list {
        //     let basic_block = graph
        //         .basic_blocks
        //         .shift_remove_index(i)
        //         .expect("there should be a BasicBlock in CFG");
        //     let mut basic_block = basic_block.borrow_mut();

        //     assert!(
        //         !basic_block.reachable(),
        //         "reachable basic blocks should not be eliminated"
        //     );

        //     basic_block.previous.clear();
        //     basic_block.terminator = Terminator::None;

        //     changed |= true;
        // }

        // changed
        false
    }
}

#[cfg(test)]
mod test {
    use super::ControlFlowGraph;

    #[test]
    fn preserve_jump() {
        let bytecode = &[
            156, 6, 120, 15, 0, 0, 0, 153, 0, 155, 118, 0, 0, 0, 0, 147, 148,
        ];

        let graph = ControlFlowGraph::generate(bytecode, &[]);

        let actual = graph.finalize();

        assert_eq!(bytecode, actual.as_slice());
    }

    #[test]
    fn preserve_jump_table() {
        let bytecode = &[
            193, 68, 0, 18, 67, 1, 72, 1, 140, 1, 76, 153, 2, 18, 155, 6, 17, 118, 38, 0, 0, 0,
            155, 5, 17, 118, 38, 0, 0, 0, 126, 16, 118, 38, 0, 0, 0, 16, 151, 153, 3, 18, 71, 1,
            143, 0, 152, 155, 152, 120, 55, 0, 0, 0, 124, 123, 71, 0, 0, 0, 1, 0, 0, 0, 68, 0, 0,
            0, 152, 147, 148, 18, 152, 147, 148,
        ];

        let graph = ControlFlowGraph::generate(bytecode, &[]);

        println!("{graph:?}");

        let actual = graph.finalize();

        assert_eq!(bytecode, actual.as_slice());
    }
}
