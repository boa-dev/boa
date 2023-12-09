//! TODO: doc

#![allow(dead_code)]
#![allow(missing_debug_implementations)]

mod basic_block;

use std::{fmt::Debug, rc::Rc};

use indexmap::IndexSet;
use rustc_hash::FxHashMap;

use crate::vm::{CodeBlock, Handler, Instruction, InstructionIterator, Opcode};

use self::basic_block::BasicBlockFlags;
pub use self::basic_block::{BasicBlock, RcBasicBlock, WeakBasicBlock};

/// TODO: doc
#[derive(Default, Clone)]
pub enum Terminator {
    /// TODO: doc
    #[default]
    None,

    /// TODO: doc
    JumpUnconditional {
        /// TODO: doc
        opcode: Opcode,
        /// TODO: doc
        target: RcBasicBlock,
    },

    /// TODO: doc
    JumpConditional {
        /// TODO: doc
        opcode: Opcode,
        /// TODO: doc
        no: RcBasicBlock,
        /// TODO: doc
        yes: RcBasicBlock,
    },

    /// TODO: doc
    TemplateLookup {
        /// TODO: doc
        no: RcBasicBlock,

        /// TODO: doc
        yes: RcBasicBlock,

        /// TODO: doc
        site: u64,
    },

    /// TODO: doc
    Return,
}

impl Terminator {
    /// Check if [`Terminator::None`].
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Terminator::None)
    }

    /// Check if [`Terminator::Jump`].
    #[must_use]
    pub fn is_jump(&self) -> bool {
        matches!(
            self,
            Terminator::JumpUnconditional { .. } | Terminator::JumpConditional { .. }
        )
    }

    /// Check if unconditional [`Terminator::Jump`].
    #[must_use]
    pub fn is_unconditional_jump(&self) -> bool {
        matches!(self, Terminator::JumpUnconditional { .. })
    }

    /// Check if conditional [`Terminator::Jump`].
    #[must_use]
    pub fn is_conditional_jump(&self) -> bool {
        matches!(self, Terminator::JumpConditional { .. })
    }
}

/// TODO: doc
pub struct ControlFlowGraph {
    basic_block_start: RcBasicBlock,
    basic_blocks: IndexSet<RcBasicBlock>,
}

impl Debug for ControlFlowGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BasicBlocks:")?;

        let mut seen = FxHashMap::default();
        let index_from_basic_block = |bb: &RcBasicBlock| {
            for (i, basic_block) in self.basic_blocks.iter().enumerate() {
                if basic_block == bb {
                    return i;
                }
            }

            unreachable!("There should be a basic block")
        };

        let mut index = 0;
        for basic_block in &self.basic_blocks {
            if seen.contains_key(&basic_block.as_ptr()) {
                continue;
            }
            seen.insert(basic_block.as_ptr(), index);

            let basic_block = basic_block.borrow();

            write!(
                f,
                "    B{index}: -- {}reachable",
                if basic_block.reachable() { "" } else { "not " }
            )?;

            if !basic_block.predecessors.is_empty() {
                write!(f, " -- predecessors ")?;
                for predecessor in &basic_block.predecessors {
                    if let Some(predecessor) = predecessor.upgrade() {
                        let index = index_from_basic_block(&predecessor);
                        write!(f, "B{index}, ")?;
                    }
                }
            }

            let successors = basic_block.successors();
            if !successors.is_empty() {
                write!(f, " -- successors ")?;
                for successor in &successors {
                    let index = index_from_basic_block(successor);
                    write!(f, "B{index}, ")?;
                }
            }

            if let Some(handler) = &basic_block.handler {
                let index = index_from_basic_block(handler);
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
                        let target = index_from_basic_block(target);
                        write!(f, "{} B{target}", opcode.as_str())?;
                    }
                    Terminator::JumpConditional { opcode, no: _, yes } => {
                        let target = index_from_basic_block(yes);
                        write!(f, "{} B{target}", opcode.as_str())?;
                    }
                    Terminator::TemplateLookup {
                        no: _,
                        yes,
                        site: _,
                    } => {
                        let target = index_from_basic_block(yes);
                        write!(f, "TemplateLookup B{target}")?;
                    }
                    Terminator::Return => {
                        write!(f, "Return")?;
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
    pub fn generate(bytecode: &[u8], handlers: &[Handler]) -> Self {
        let leaders = Self::leaders(bytecode, handlers);
        let block_count = leaders.len();

        let mut basic_blocks = IndexSet::with_capacity(block_count);
        for _ in 0..block_count {
            basic_blocks.insert(RcBasicBlock::default());
        }

        let basic_block_from_bytecode_position = |address: u32| {
            let index = leaders
                .iter()
                .position(|x| *x == address)
                .expect("There should be a basic block");

            basic_blocks[index].clone()
        };

        let mut iter = InstructionIterator::new(bytecode);
        for (i, leader) in leaders
            .iter()
            .map(|x| *x as usize)
            .enumerate()
            .skip(1)
            .map(|(i, leader)| (i - 1, leader))
        {
            let this = basic_blocks[i].clone();

            let handler = handlers
                .iter()
                .rev()
                .find(|handler| handler.contains(iter.pc() as u32));
            if let Some(handler) = handler {
                let handler = basic_block_from_bytecode_position(handler.handler());

                this.borrow_mut().handler = Some(handler);
            }

            let mut bytecode = Vec::new();
            let mut terminator = Terminator::None;
            while let Some((_, _, instruction)) = iter.next() {
                match instruction {
                    Instruction::Return => {
                        terminator = Terminator::Return;
                    }
                    Instruction::Jump { address } | Instruction::Default { address } => {
                        let target = basic_block_from_bytecode_position(address);

                        target.borrow_mut().predecessors.push(this.downgrade());

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
                        let no = basic_blocks[i + 1].clone();

                        yes.borrow_mut().predecessors.push(this.downgrade());
                        no.borrow_mut().predecessors.push(this.downgrade());

                        terminator = Terminator::TemplateLookup { no, yes, site };
                    }
                    instruction => {
                        if let Some(address) = is_jump_kind_instruction(&instruction) {
                            let yes = basic_block_from_bytecode_position(address);
                            let no = basic_blocks[i + 1].clone();

                            yes.borrow_mut().predecessors.push(this.downgrade());
                            no.borrow_mut().predecessors.push(this.downgrade());

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

            let mut basic_block = this.borrow_mut();
            basic_block.instructions = bytecode;
            basic_block.terminator = terminator;
        }

        Self {
            basic_block_start: basic_blocks[0].clone(),
            basic_blocks,
        }
    }

    /// Remove [`BasicBlock`].
    pub fn remove(&mut self, basic_block: &RcBasicBlock) {
        self.basic_blocks.shift_remove(basic_block);
    }

    /// Get [`BasicBlock`] index.
    #[must_use]
    pub fn get_index(&self, basic_block: &RcBasicBlock) -> usize {
        self.basic_blocks
            .get_index_of(basic_block)
            .expect("there should be a BasicBlock in CFG")
    }

    /// Finalize bytecode.
    #[must_use]
    pub fn finalize(self) -> Vec<u8> {
        let index_from_basic_block = |bb: &RcBasicBlock| {
            for (i, basic_block) in self.basic_blocks.iter().enumerate() {
                if Rc::ptr_eq(basic_block, bb) {
                    return i;
                }
            }

            unreachable!("There should be a basic block")
        };

        let mut results = Vec::new();
        let mut labels = Vec::new();
        let mut blocks = Vec::with_capacity(self.basic_blocks.len());

        for basic_block in &self.basic_blocks {
            let basic_block = basic_block.borrow();

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

                    let target = index_from_basic_block(target);
                    labels.push((start as u32, target));
                }
                Terminator::JumpConditional { opcode, no: _, yes } => {
                    results.extend_from_slice(&[*opcode as u8]);
                    let start = results.len();
                    results.extend_from_slice(&[0, 0, 0, 0]);

                    let target = index_from_basic_block(yes);
                    labels.push((start as u32, target));
                }
                Terminator::TemplateLookup { yes, site, .. } => {
                    results.extend_from_slice(&[Opcode::TemplateLookup as u8]);
                    let start = results.len();
                    results.extend_from_slice(&[0, 0, 0, 0]);
                    results.extend_from_slice(&site.to_ne_bytes());

                    let target = index_from_basic_block(yes);
                    labels.push((start as u32, target));
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

impl Drop for ControlFlowGraph {
    fn drop(&mut self) {
        // NOTE: Untie BasicBlock nodes, so they can be deallocated.
        for basic_block in &self.basic_blocks {
            *basic_block.borrow_mut() = BasicBlock::default();
        }
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
    pub fn perform(graph: &mut ControlFlowGraph) -> bool {
        let mut changed = false;
        for basic_block_ptr in &graph.basic_blocks {
            {
                let mut basic_block = basic_block_ptr.borrow_mut();

                #[allow(clippy::single_match)]
                match basic_block.terminator.clone() {
                    Terminator::JumpConditional { no, yes, .. } => {
                        if no == yes {
                            basic_block.insert_last(Instruction::Pop);
                            basic_block.terminator = Terminator::JumpUnconditional {
                                opcode: Opcode::Jump,
                                target: yes,
                            };

                            changed |= true;
                        }
                    }
                    _ => {}
                }
            }
        }
        changed
    }
}

/// TODO: doc
#[derive(Clone, Copy)]
pub struct GraphEliminateUnreachableBasicBlocks;

impl GraphEliminateUnreachableBasicBlocks {
    /// TODO: doc
    pub fn perform(graph: &mut ControlFlowGraph) -> bool {
        let mut changed = false;

        let mut stack = vec![graph.basic_block_start.clone()];
        while let Some(basic_block_ptr) = stack.pop() {
            let mut basic_block = basic_block_ptr.borrow_mut();
            if basic_block.reachable() {
                break;
            }
            basic_block.flags |= BasicBlockFlags::REACHABLE;
            basic_block.next(&mut stack);

            // println!("{:p} -- {}", basic_block_ptr.as_ptr(), basic_block.reachable());
        }

        assert!(
            graph.basic_block_start.borrow().reachable(),
            "start basic block node should always be reachable"
        );

        let mut delete_list = Vec::new();
        for (i, basic_block) in graph.basic_blocks.iter().enumerate().rev() {
            if !basic_block.borrow().reachable() {
                delete_list.push(i);
            }
        }

        // println!("{delete_list:?}");

        for i in delete_list {
            let basic_block = graph
                .basic_blocks
                .shift_remove_index(i)
                .expect("there should be a BasicBlock in CFG");
            let mut basic_block = basic_block.borrow_mut();

            assert!(
                !basic_block.reachable(),
                "reachable basic blocks should not be eliminated"
            );

            basic_block.predecessors.clear();
            basic_block.terminator = Terminator::None;

            changed |= true;
        }

        changed
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
}
