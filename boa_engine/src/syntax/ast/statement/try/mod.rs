use crate::syntax::ast::statement::{Block, Statement};
use boa_interner::{Interner, ToInternedString};

use super::{declaration::Binding, ContainsSymbol};

#[cfg(test)]
mod tests;

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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Try {
    block: Block,
    catch: Option<Catch>,
    finally: Option<Finally>,
}

impl Try {
    /// Creates a new `Try` AST node.
    pub(in crate::syntax) fn new(
        block: Block,
        catch: Option<Catch>,
        finally: Option<Finally>,
    ) -> Self {
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
    pub fn block(&self) -> &Block {
        &self.block
    }

    /// Gets the `catch` block, if any.
    pub fn catch(&self) -> Option<&Catch> {
        self.catch.as_ref()
    }

    /// Gets the `finally` block, if any.
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

    /// Implements the display formatting with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
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

impl ToInternedString for Try {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Try> for Statement {
    fn from(try_catch: Try) -> Self {
        Self::Try(try_catch)
    }
}

/// Catch block.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Catch {
    parameter: Option<Binding>,
    block: Block,
}

impl Catch {
    /// Creates a new catch block.
    pub(in crate::syntax) fn new(parameter: Option<Binding>, block: Block) -> Self {
        Self { parameter, block }
    }

    /// Gets the parameter of the catch block.
    pub fn parameter(&self) -> Option<&Binding> {
        self.parameter.as_ref()
    }

    /// Retrieves the catch execution block.
    pub fn block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.block
            .statements()
            .iter()
            .any(Statement::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.block
            .statements()
            .iter()
            .any(|stmt| stmt.contains(symbol))
    }

    /// Implements the display formatting with indentation.
    pub(super) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
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

impl ToInternedString for Catch {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

/// Finally block.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Finally {
    block: Block,
}

impl Finally {
    /// Gets the finally block.
    pub fn block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.block
            .statements()
            .iter()
            .any(Statement::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.block
            .statements()
            .iter()
            .any(|stmt| stmt.contains(symbol))
    }

    /// Implements the display formatting with indentation.
    pub(super) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            " finally {}",
            self.block.to_indented_string(interner, indentation)
        )
    }
}

impl<T> From<T> for Finally
where
    T: Into<Block>,
{
    fn from(block: T) -> Self {
        Self {
            block: block.into(),
        }
    }
}
