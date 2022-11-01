//! Error handling statements

use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    declaration::Binding,
    statement::{Block, Statement},
    StatementListItem,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

use super::ContainsSymbol;

/// The `try...catch` statement marks a block of statements to try and specifies a response
/// should an exception be thrown.
///
/// The `try` statement consists of a `try`-block, which contains one or more statements. `{}`
/// must always be used, even for single statements. At least one `catch`-block, or a
/// `finally`-block, must be present.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-TryStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Try {
    block: Block,
    catch: Option<Catch>,
    finally: Option<Finally>,
}

impl Try {
    /// Creates a new `Try` AST node.
    #[inline]
    pub fn new(block: Block, catch: Option<Catch>, finally: Option<Finally>) -> Self {
        assert!(
            catch.is_some() || finally.is_some(),
            "one of catch or finally must be pressent"
        );

        Self {
            block,
            catch,
            finally,
        }
    }

    /// Gets the `try` block.
    #[inline]
    pub fn block(&self) -> &Block {
        &self.block
    }

    /// Gets the `catch` block, if any.
    #[inline]
    pub fn catch(&self) -> Option<&Catch> {
        self.catch.as_ref()
    }

    /// Gets the `finally` block, if any.
    #[inline]
    pub fn finally(&self) -> Option<&Block> {
        self.finally.as_ref().map(Finally::block)
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.block.contains_arguments()
            || matches!(self.catch, Some(ref catch) if catch.contains_arguments())
            || matches!(self.finally, Some(ref finally) if finally.contains_arguments())
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.block.contains(symbol)
            || matches!(self.catch, Some(ref catch) if catch.contains(symbol))
            || matches!(self.finally, Some(ref finally) if finally.contains(symbol))
    }
}

impl ToIndentedString for Try {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = format!(
            "{}try {}",
            "    ".repeat(indentation),
            self.block.to_indented_string(interner, indentation)
        );

        if let Some(ref catch) = self.catch {
            buf.push_str(&catch.to_indented_string(interner, indentation));
        }

        if let Some(ref finally) = self.finally {
            buf.push_str(&finally.to_indented_string(interner, indentation));
        }
        buf
    }
}

impl From<Try> for Statement {
    #[inline]
    fn from(try_catch: Try) -> Self {
        Self::Try(try_catch)
    }
}

impl VisitWith for Try {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_block(&self.block));
        if let Some(catch) = &self.catch {
            try_break!(visitor.visit_catch(catch));
        }
        if let Some(finally) = &self.finally {
            try_break!(visitor.visit_finally(finally));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_block_mut(&mut self.block));
        if let Some(catch) = &mut self.catch {
            try_break!(visitor.visit_catch_mut(catch));
        }
        if let Some(finally) = &mut self.finally {
            try_break!(visitor.visit_finally_mut(finally));
        }
        ControlFlow::Continue(())
    }
}

/// Catch block.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Catch {
    parameter: Option<Binding>,
    block: Block,
}

impl Catch {
    /// Creates a new catch block.
    #[inline]
    pub fn new(parameter: Option<Binding>, block: Block) -> Self {
        Self { parameter, block }
    }

    /// Gets the parameter of the catch block.
    #[inline]
    pub fn parameter(&self) -> Option<&Binding> {
        self.parameter.as_ref()
    }

    /// Retrieves the catch execution block.
    #[inline]
    pub fn block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.block
            .statement_list()
            .statements()
            .iter()
            .any(StatementListItem::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.block
            .statement_list()
            .statements()
            .iter()
            .any(|stmt| stmt.contains(symbol))
    }
}

impl ToIndentedString for Catch {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = " catch".to_owned();
        if let Some(ref param) = self.parameter {
            buf.push_str(&format!("({})", param.to_interned_string(interner)));
        }
        buf.push_str(&format!(
            " {}",
            self.block.to_indented_string(interner, indentation)
        ));

        buf
    }
}

impl VisitWith for Catch {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(binding) = &self.parameter {
            try_break!(visitor.visit_binding(binding));
        }
        visitor.visit_block(&self.block)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(binding) = &mut self.parameter {
            try_break!(visitor.visit_binding_mut(binding));
        }
        visitor.visit_block_mut(&mut self.block)
    }
}

/// Finally block.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Finally {
    block: Block,
}

impl Finally {
    /// Gets the finally block.
    #[inline]
    pub fn block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.block
            .statement_list()
            .statements()
            .iter()
            .any(StatementListItem::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.block
            .statement_list()
            .statements()
            .iter()
            .any(|stmt| stmt.contains(symbol))
    }
}

impl ToIndentedString for Finally {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            " finally {}",
            self.block.to_indented_string(interner, indentation)
        )
    }
}

impl From<Block> for Finally {
    #[inline]
    fn from(block: Block) -> Self {
        Self { block }
    }
}

impl VisitWith for Finally {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_block(&self.block)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_block_mut(&mut self.block)
    }
}
