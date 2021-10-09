use crate::{
    syntax::{
        ast::{
            node::{self, Identifier},
            Keyword, Position, Punctuator,
        },
        lexer::TokenKind,
        parser::{
            statement::{
                block::Block, ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern,
            },
            Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use rustc_hash::FxHashSet;
use std::io::Read;

/// Catch parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-Catch
#[derive(Debug, Clone, Copy)]
pub(super) struct Catch<const YIELD: bool, const AWAIT: bool, const RETURN: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for Catch<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = node::Catch;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Catch", "Parsing");
        cursor.expect(Keyword::Catch, "try statement")?;
        let catch_param = if cursor.next_if(Punctuator::OpenParen)?.is_some() {
            let catch_param = CatchParameter::<YIELD, AWAIT>.parse(cursor)?;

            cursor.expect(Punctuator::CloseParen, "catch in try statement")?;
            Some(catch_param)
        } else {
            None
        };

        let mut set = FxHashSet::default();
        let idents = match &catch_param {
            Some(node::Declaration::Identifier { ident, .. }) => vec![ident.as_ref()],
            Some(node::Declaration::Pattern(p)) => p.idents(),
            _ => vec![],
        };

        // It is a Syntax Error if BoundNames of CatchParameter contains any duplicate elements.
        // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
        for ident in idents {
            if !set.insert(ident) {
                // FIXME: pass correct position once #1295 lands
                return Err(ParseError::general(
                    "duplicate identifier",
                    Position::new(1, 1),
                ));
            }
        }

        // Catch block
        let catch_block = Block::<YIELD, AWAIT, RETURN>.parse(cursor)?;

        // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the LexicallyDeclaredNames of Block.
        // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the VarDeclaredNames of Block.
        // https://tc39.es/ecma262/#sec-try-statement-static-semantics-early-errors

        // FIXME: `lexically_declared_names` only holds part of LexicallyDeclaredNames of the
        // Block e.g. function names are *not* included but should be.
        let lexically_declared_names = catch_block.lexically_declared_names();
        let var_declared_names = catch_block.var_declared_named();

        for ident in set {
            // FIXME: pass correct position once #1295 lands
            if lexically_declared_names.contains(ident) {
                return Err(ParseError::general(
                    "identifier redeclared",
                    Position::new(1, 1),
                ));
            }
            if var_declared_names.contains(ident) {
                return Err(ParseError::general(
                    "identifier redeclared",
                    Position::new(1, 1),
                ));
            }
        }

        let catch_node = node::Catch::new::<_, node::Declaration, _>(catch_param, catch_block);
        Ok(catch_node)
    }
}

/// CatchParameter parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-CatchParameter
#[derive(Debug, Clone, Copy)]
pub(super) struct CatchParameter<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for CatchParameter<YIELD, AWAIT>
where
    R: Read,
{
    type Output = node::Declaration;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let pat = ObjectBindingPattern::<true, YIELD, AWAIT>.parse(cursor)?;

                Ok(node::Declaration::new_with_object_pattern(pat, None))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let pat = ArrayBindingPattern::<true, YIELD, AWAIT>.parse(cursor)?;
                Ok(node::Declaration::new_with_array_pattern(pat, None))
            }
            TokenKind::Identifier(_) => {
                let ident = BindingIdentifier::<YIELD, AWAIT>
                    .parse(cursor)
                    .map(Identifier::from)?;
                Ok(node::Declaration::new_with_identifier(ident, None))
            }
            _ => Err(ParseError::unexpected(token.clone(), None)),
        }
    }
}
