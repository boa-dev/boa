use crate::{
    bytecompiler::{ByteCompiler, Label},
    vm::Opcode,
};
use boa_interner::Sym;
use std::mem::size_of;

#[derive(Debug, Clone)]
pub(crate) struct JumpControlInfo {
    label: Option<Sym>,
    start_address: u32,
    kind: JumpControlInfoKind,
    breaks: Vec<Label>,
    try_continues: Vec<Label>,
    in_catch: bool,
    has_finally: bool,
    finally_start: Option<Label>,
    for_of_in_loop: bool,
    decl_envs: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum JumpControlInfoKind {
    Loop,
    Switch,
    Try,
    LabelledBlock,
}

// ---- Creation Methods ---- //

impl JumpControlInfo {
    pub(crate) const fn new() -> Self {
        Self {
            label: None,
            start_address: u32::MAX,
            kind: JumpControlInfoKind::Loop,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            in_catch: false,
            has_finally: false,
            finally_start: None,
            for_of_in_loop: false,
            decl_envs: 0,
        }
    }

    pub(crate) const fn with_label(mut self, label: Option<Sym>) -> Self {
        self.label = label;
        self
    }

    pub(crate) const fn with_start_address(mut self, address: u32) -> Self {
        self.start_address = address;
        self
    }

    pub(crate) const fn with_kind(mut self, kind: JumpControlInfoKind) -> Self {
        self.kind = kind;
        self
    }

    pub(crate) const fn with_has_finally(mut self, value: bool) -> Self {
        self.has_finally = value;
        self
    }

    pub(crate) const fn with_for_of_in_loop(mut self, value: bool) -> Self {
        self.for_of_in_loop = value;
        self
    }
}

impl JumpControlInfo {
    pub(crate) const fn label(&self) -> Option<Sym> {
        self.label
    }

    pub(crate) fn set_label(&mut self, label: Option<Sym>) {
        assert!(self.label.is_none());
        self.label = label;
    }

    pub(crate) const fn start_address(&self) -> u32 {
        self.start_address
    }

    pub(crate) fn set_start_address(&mut self, start_address: u32) {
        self.start_address = start_address;
    }

    pub(crate) const fn kind(&self) -> JumpControlInfoKind {
        self.kind
    }

    pub(crate) fn set_in_catch(&mut self, value: bool) {
        self.in_catch = value;
    }

    pub(crate) const fn in_catch(&self) -> bool {
        self.in_catch
    }

    pub(crate) const fn has_finally(&self) -> bool {
        self.has_finally
    }

    pub(crate) fn set_finally_start(&mut self, label: Label) {
        self.finally_start = Some(label);
    }

    pub(crate) const fn finally_start(&self) -> Option<Label> {
        self.finally_start
    }

    pub(crate) const fn for_of_in_loop(&self) -> bool {
        self.for_of_in_loop
    }

    pub(crate) const fn decl_envs(&self) -> u32 {
        self.decl_envs
    }

    pub(crate) fn inc_decl_envs(&mut self) {
        self.decl_envs += 1;
    }

    pub(crate) fn dec_decl_envs(&mut self) {
        self.decl_envs -= 1;
    }

    pub(crate) fn push_break_label(&mut self, break_label: Label) {
        self.breaks.push(break_label);
    }

    pub(crate) fn push_try_continue_label(&mut self, try_continue_label: Label) {
        self.try_continues.push(try_continue_label);
    }
}

impl ByteCompiler<'_, '_> {
    pub(crate) fn push_new_jump_control(&mut self) {
        self.jump_info.push(JumpControlInfo::new());
    }

    /*
    pub(crate) fn inc_jump_control_decl_envs(&mut self) {
        assert!(!self.jump_info.is_empty());
        self.jump_info
            .last_mut()
            .expect("JumpInfo must exist")
            .inc_decl_envs();
    }
    */

    pub(crate) fn set_jump_control_label(&mut self, label: Option<Sym>) {
        assert!(!self.jump_info.is_empty());
        self.jump_info
            .last_mut()
            .expect("JumpInfo must exist")
            .set_label(label);
    }

    pub(crate) fn set_jump_control_start_address(&mut self, start_address: u32) {
        assert!(!self.jump_info.is_empty());
        self.jump_info
            .last_mut()
            .expect("JumpInfo must exist")
            .set_start_address(start_address);
    }

    pub(crate) fn set_jump_control_finally_start(&mut self, start: Label) {
        if !self.jump_info.is_empty() {
            let info = self
                .jump_info
                .last_mut()
                .expect("must have try control label");
            assert!(info.kind == JumpControlInfoKind::Try);
            info.set_finally_start(start);
        }
    }

    pub(crate) fn set_jump_control_catch_start(&mut self, value: bool) {
        if !self.jump_info.is_empty() {
            let info = self
                .jump_info
                .last_mut()
                .expect("must have try control label");
            assert!(info.kind == JumpControlInfoKind::Try);
            info.set_in_catch(value);
        }
    }

    /// Emits the `PushDeclarativeEnvironment` and updates the current jump info to track environments
    pub(crate) fn emit_and_track_decl_env(&mut self) -> (Label, Label) {
        let pushed_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        if !self.jump_info.is_empty() {
            let current_jump_info = self
                .jump_info
                .last_mut()
                .expect("Jump info must exist as the vector is not empty");
            current_jump_info.inc_decl_envs();
        }
        pushed_env
    }

    pub(crate) fn emit_and_track_pop_env(&mut self) {
        self.emit_opcode(Opcode::PopEnvironment);
        if !self.jump_info.is_empty() {
            let current_info = self.jump_info.last_mut().expect("JumpInfo must exist");
            current_info.dec_decl_envs();
        }
    }

    pub(crate) fn push_loop_control_info(&mut self, label: Option<Sym>, start_address: u32) {
        let new_info = JumpControlInfo::new()
            .with_label(label)
            .with_start_address(start_address);
        self.jump_info.push(new_info);
    }

    pub(crate) fn push_loop_control_info_for_of_in_loop(
        &mut self,
        label: Option<Sym>,
        start_address: u32,
    ) {
        let new_info = JumpControlInfo::new()
            .with_label(label)
            .with_start_address(start_address)
            .with_for_of_in_loop(true);
        self.jump_info.push(new_info);
    }

    pub(crate) fn pop_loop_control_info(&mut self) {
        let loop_info = self.jump_info.pop().expect("no jump information found");

        assert!(loop_info.kind == JumpControlInfoKind::Loop);

        for label in loop_info.breaks {
            self.patch_jump(label);
        }

        for label in loop_info.try_continues {
            self.patch_jump_with_target(label, loop_info.start_address);
        }
    }

    pub(crate) fn push_switch_control_info(&mut self, label: Option<Sym>, start_address: u32) {
        let new_info = JumpControlInfo::new()
            .with_kind(JumpControlInfoKind::Switch)
            .with_label(label)
            .with_start_address(start_address);
        self.jump_info.push(new_info);
    }

    pub(crate) fn pop_switch_control_info(&mut self) {
        let info = self.jump_info.pop().expect("no jump information found");

        assert!(info.kind == JumpControlInfoKind::Switch);

        for label in info.breaks {
            self.patch_jump(label);
        }
    }

    pub(crate) fn push_try_control_info(&mut self, has_finally: bool) {
        if !self.jump_info.is_empty() {
            let start_address = self
                .jump_info
                .last()
                .expect("no jump information found")
                .start_address();

            let new_info = JumpControlInfo::new()
                .with_kind(JumpControlInfoKind::Try)
                .with_start_address(start_address)
                .with_has_finally(has_finally);

            self.jump_info.push(new_info);
        }
    }

    pub(crate) fn pop_try_control_info(&mut self, finally_start_address: Option<u32>) {
        if !self.jump_info.is_empty() {
            let mut info = self.jump_info.pop().expect("no jump information found");

            assert!(info.kind == JumpControlInfoKind::Try);

            let mut breaks = Vec::with_capacity(info.breaks.len());

            if let Some(finally_start_address) = finally_start_address {
                for label in info.try_continues {
                    if label.index < finally_start_address {
                        self.patch_jump_with_target(label, finally_start_address);
                    } else {
                        self.patch_jump_with_target(label, info.start_address);
                    }
                }

                for label in info.breaks {
                    if label.index < finally_start_address {
                        self.patch_jump_with_target(label, finally_start_address);
                        let Label { mut index } = label;
                        index -= size_of::<Opcode>() as u32;
                        index -= size_of::<u32>() as u32;
                        breaks.push(Label { index });
                    } else {
                        breaks.push(label);
                    }
                }
                if let Some(jump_info) = self.jump_info.last_mut() {
                    jump_info.breaks.append(&mut breaks);
                }
            } else if let Some(jump_info) = self.jump_info.last_mut() {
                jump_info.breaks.append(&mut info.breaks);
                jump_info.try_continues.append(&mut info.try_continues);
            }
        }
    }

    pub(crate) fn push_labelled_block_control_info(&mut self, label: Sym, start_address: u32) {
        let new_info = JumpControlInfo::new()
            .with_kind(JumpControlInfoKind::LabelledBlock)
            .with_label(Some(label))
            .with_start_address(start_address);
        self.jump_info.push(new_info);
    }

    pub(crate) fn pop_labelled_block_control_info(&mut self) {
        let info = self.jump_info.pop().expect("no jump information found");

        assert!(info.kind == JumpControlInfoKind::LabelledBlock);

        self.emit_opcode(Opcode::PopEnvironment);

        for label in info.breaks {
            self.patch_jump(label);
        }

        for label in info.try_continues {
            self.patch_jump_with_target(label, info.start_address);
        }
    }
}
