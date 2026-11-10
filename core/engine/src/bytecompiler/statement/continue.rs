use crate::bytecompiler::{
    ByteCompiler,
    jump_control::{JumpRecord, JumpRecordAction, JumpRecordKind},
};
use boa_ast::statement::Continue;
use boa_interner::Sym;

impl ByteCompiler<'_> {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn compile_continue(&mut self, node: Continue, _use_expr: bool) {
        let actions = self.continue_jump_record_actions(node);

        JumpRecord::new(JumpRecordKind::Continue, actions)
            .perform_actions(Self::DUMMY_ADDRESS, self);
    }

    fn continue_jump_record_actions(&self, node: Continue) -> Vec<JumpRecordAction> {
        let target = node.label().and_then(|label| self.continue_target(label));
        let mut actions = Vec::default();
        for (i, info) in self.jump_info.iter().enumerate().rev() {
            let count = self.jump_info_open_environment_count(i);
            actions.push(JumpRecordAction::PopEnvironments { count });

            if !info.in_finally()
                && let Some((finally_throw_flag, finally_throw_index)) = info.finally_throw
            {
                actions.push(JumpRecordAction::HandleFinally {
                    index: info.jumps.len() as u32,
                    finally_throw_flag,
                    finally_throw_index,
                });
                actions.push(JumpRecordAction::Transfer { index: i as u32 });
            }

            if let Some(target) = target {
                if i == target {
                    actions.push(JumpRecordAction::Transfer { index: i as u32 });
                    break;
                }
            } else if info.is_loop() {
                actions.push(JumpRecordAction::Transfer { index: i as u32 });
                break;
            }

            if info.iterator_loop() {
                actions.push(JumpRecordAction::CloseIterator {
                    r#async: info.for_await_of_loop(),
                });
            }
        }

        actions.reverse();
        actions
    }

    /// Finds the index of the jump info that a `continue label` targets.
    ///
    /// A loop can have more than one label (`a: b: for (;;)`), but only the innermost one is
    /// attached to the loop's own jump info; the outer ones are separate labelled-statement infos
    /// pushed right before it. A `continue` to any of them has to continue the loop itself.
    ///
    /// In the spec, [LabelledEvaluation of a `LabelledStatement`][labelled] appends its label to
    /// `labelSet` and passes the set down to the labelled item, so a loop under `a: b:` gets
    /// the label set « a, b ». [LoopContinues][loop-continues] then returns `true` for a
    /// `continue` completion whose target is any label in that set (step 4), which means
    /// `continue a` continues the loop the same way `continue b` does.
    ///
    /// [labelled]: https://tc39.es/ecma262/#sec-labelled-statements-runtime-semantics-labelledevaluation
    /// [loop-continues]: https://tc39.es/ecma262/#sec-loopcontinues
    fn continue_target(&self, label: Sym) -> Option<usize> {
        let found = self
            .jump_info
            .iter()
            .rposition(|info| info.label() == Some(label))?;
        let mut index = found;
        while self
            .jump_info
            .get(index)
            .is_some_and(|info| info.is_labelled() && !info.is_loop())
        {
            index += 1;
        }
        match self.jump_info.get(index) {
            Some(info) if info.is_loop() => Some(index),
            _ => Some(found),
        }
    }
}
