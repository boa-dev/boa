use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    ops::Deref,
    rc::{Rc, Weak},
};

use bitflags::bitflags;

use crate::vm::Instruction;

use super::Terminator;

bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct BasicBlockFlags: u8 {
        const REACHABLE = 0b0000_0001;
    }
}

/// TODO: doc
#[derive(Default, Clone)]
pub struct BasicBlock {
    pub(crate) predecessors: Vec<WeakBasicBlock>,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) terminator: Terminator,
    pub(crate) handler: Option<RcBasicBlock>,

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

    pub(crate) fn successors(&self) -> Vec<RcBasicBlock> {
        match &self.terminator {
            Terminator::None => vec![],
            Terminator::JumpUnconditional { target, .. } => {
                vec![target.clone()]
            }
            Terminator::JumpConditional { no, yes, .. }
            | Terminator::TemplateLookup { no, yes, .. } => {
                vec![no.clone(), yes.clone()]
            }
            Terminator::Return => Vec::new(),
        }
    }

    pub(crate) fn next(&self, nexts: &mut Vec<RcBasicBlock>) {
        match &self.terminator {
            Terminator::None | Terminator::Return => {}
            Terminator::JumpUnconditional { target, .. } => {
                nexts.push(target.clone());
            }
            Terminator::JumpConditional { no, yes, .. }
            | Terminator::TemplateLookup { no, yes, .. } => {
                nexts.push(no.clone());
                nexts.push(yes.clone());
            }
        }
    }
}

/// Reference counted [`BasicBlock`] with interor mutability.
#[derive(Default, Clone)]
pub struct RcBasicBlock {
    inner: Rc<RefCell<BasicBlock>>,
}

impl From<Rc<RefCell<BasicBlock>>> for RcBasicBlock {
    fn from(inner: Rc<RefCell<BasicBlock>>) -> Self {
        Self { inner }
    }
}

impl Deref for RcBasicBlock {
    type Target = Rc<RefCell<BasicBlock>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialEq<RcBasicBlock> for RcBasicBlock {
    fn eq(&self, other: &RcBasicBlock) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for RcBasicBlock {}

impl Hash for RcBasicBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.as_ptr() as usize).hash(state);
    }
}

impl RcBasicBlock {
    /// TODO: doc
    #[must_use]
    pub fn downgrade(&self) -> WeakBasicBlock {
        WeakBasicBlock::from(Rc::downgrade(&self.inner))
    }
}

/// Reference counted [`BasicBlock`] with interor mutability.
#[derive(Default, Clone)]
pub struct WeakBasicBlock {
    inner: Weak<RefCell<BasicBlock>>,
}

impl From<Weak<RefCell<BasicBlock>>> for WeakBasicBlock {
    fn from(inner: Weak<RefCell<BasicBlock>>) -> Self {
        Self { inner }
    }
}

impl Deref for WeakBasicBlock {
    type Target = Weak<RefCell<BasicBlock>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialEq<WeakBasicBlock> for WeakBasicBlock {
    fn eq(&self, other: &WeakBasicBlock) -> bool {
        Weak::ptr_eq(&self.inner, &other.inner)
    }
}

impl WeakBasicBlock {
    /// TODO: doc
    #[must_use]
    pub fn upgrade(&self) -> Option<RcBasicBlock> {
        Some(RcBasicBlock::from(self.inner.upgrade()?))
    }
}
