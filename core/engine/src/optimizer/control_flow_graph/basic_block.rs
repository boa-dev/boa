use std::hash::Hash;

use bitflags::bitflags;

use crate::vm::Instruction;

use super::{BasicBlockKey, Terminator};

bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct BasicBlockFlags: u8 {
        const REACHABLE = 0b0000_0001;
    }
}

/// TODO: doc
#[derive(Default, Clone)]
pub struct BasicBlock {
    pub(crate) previous: Vec<BasicBlockKey>,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) terminator: Terminator,
    pub(crate) handler: Option<BasicBlockKey>,

    pub(crate) flags: BasicBlockFlags,
}

impl BasicBlock {
    /// Get nth instruction in the [`BasicBlock`].
    pub(crate) fn get(&mut self, nth: usize) -> Option<&Instruction> {
        self.instructions.get(nth)
    }

    /// Insert nth instruction in the [`BasicBlock`].
    pub(crate) fn insert(&mut self, nth: usize, instruction: Instruction) {
        self.instructions.insert(nth, instruction);
    }

    /// Insert instruction in the last position in the [`BasicBlock`].
    pub(crate) fn insert_last(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Remove nth instruction in the [`BasicBlock`].
    pub(crate) fn remove(&mut self, nth: usize) -> Instruction {
        self.instructions.remove(nth)
    }

    /// Remove last instruction in the [`BasicBlock`].
    pub(crate) fn remove_last(&mut self) -> bool {
        self.instructions.pop().is_some()
    }

    pub(crate) fn reachable(&self) -> bool {
        self.flags.contains(BasicBlockFlags::REACHABLE)
    }

    pub(crate) fn next(&self) -> Vec<BasicBlockKey> {
        let mut result = Vec::new();
        self.next_into(&mut result);
        result
    }

    pub(crate) fn next_into(&self, nexts: &mut Vec<BasicBlockKey>) {
        match self.terminator {
            Terminator::None => {}
            Terminator::JumpUnconditional { target, .. } => {
                nexts.push(target);
            }
            Terminator::JumpConditional { no, yes, .. }
            | Terminator::TemplateLookup { no, yes, .. } => {
                nexts.push(no);
                nexts.push(yes);
            }
            Terminator::Return { end } => nexts.push(end),
        }
    }
}
