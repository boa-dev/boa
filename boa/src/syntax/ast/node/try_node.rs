use super::{Block, Identifier, Node};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Try {
    block: Block,
    catch: Option<Catch>,
    finally: Option<Finally>,
}

impl Try {
    /// Creates a new `Try` AST node.
    pub(in crate::syntax) fn new<B>(
        block: B,
        catch: Option<Catch>,
        finally: Option<Finally>,
    ) -> Self
    where
        B: Into<Block>,
    {
        assert!(
            catch.is_some() || finally.is_some(),
            "one of catch or finally must be pressent"
        );

        Self {
            block: block.into(),
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

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "{}try ", "    ".repeat(indentation))?;
        self.block.display(f, indentation)?;

        if let Some(ref catch) = self.catch {
            catch.display(f, indentation)?;
        }

        if let Some(ref finally) = self.finally {
            finally.display(f, indentation)?;
        }
        Ok(())
    }
}

impl fmt::Display for Try {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<Try> for Node {
    fn from(try_catch: Try) -> Self {
        Self::Try(try_catch)
    }
}

/// Catch block.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Catch {
    parameter: Option<Identifier>,
    block: Block,
}

impl Catch {
    /// Creates a new catch block.
    pub(in crate::syntax) fn new<OI, I, B>(parameter: OI, block: B) -> Self
    where
        OI: Into<Option<I>>,
        I: Into<Identifier>,
        B: Into<Block>,
    {
        Self {
            parameter: parameter.into().map(I::into),
            block: block.into(),
        }
    }

    /// Gets the parameter of the catch block.
    pub fn parameter(&self) -> Option<&str> {
        self.parameter.as_ref().map(Identifier::as_ref)
    }

    /// Retrieves the catch execution block.
    pub fn block(&self) -> &Block {
        &self.block
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        f.write_str(" catch")?;
        if let Some(ref param) = self.parameter {
            write!(f, "({})", param)?;
        }
        f.write_str(" ")?;
        self.block.display(f, indentation)
    }
}

impl fmt::Display for Catch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

/// Finally block.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Finally {
    block: Block,
}

impl Finally {
    /// Gets the finally block.
    pub fn block(&self) -> &Block {
        &self.block
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        f.write_str(" finally ")?;
        self.block.display(f, indentation)
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
