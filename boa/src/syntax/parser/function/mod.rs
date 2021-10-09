//! Function definition parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#sec-function-definitions

#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node, Punctuator, node::declaration::{Declaration, BindingPatternTypeObject},},
        lexer::{InputElement, TokenKind},
        parser::{
            expression::Initializer,
            statement::{BindingIdentifier, StatementList, ObjectBindingPattern},
            AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
use std::{collections::HashSet, io::Read};

/// Formal parameters parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameters
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FormalParameters {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameters {
    /// Creates a new `FormalParameters` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for FormalParameters
where
    R: Read,
    
{
    type Output = Box<[node::FormalParameter]>;
    
    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameters", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut params = Vec::new();
        let mut param_names = HashSet::new();

        if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseParen)
        {
            return Ok(params.into_boxed_slice());
        }
        let mut i: i32 =1; // Delete this before submitting /////////////////////////////////////////////////////////////////////

        loop {
            let mut rest_param = false;

            let position = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
            let next_params = match cursor.peek(0)? {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Spread) => {
                    rest_param = true;
                    FunctionRestParameter::new(self.allow_yield, self.allow_await).parse(cursor)?
                }
                _ => FormalParameter::new(self.allow_yield, self.allow_await).parse(cursor)?,
            };
            let next_params_iter = next_params.iter();
            
            for next_param in next_params_iter{
                std::println!("{} {}", i, next_param.name()); // Delete this before submitting /////////////////////////////////////////////////////////////////////4rcc4
                i+=1;// Delete this before submitting /////////////////////////////////////////////////////////////////////
                if param_names.contains(next_param.name()) {
                    return Err(ParseError::general("duplicate parameter name", position));
                }
                param_names.insert(Box::from(next_param.name()));
                params.push(next_param.clone());
            }

            
            if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
                == &TokenKind::Punctuator(Punctuator::CloseParen)
            {
                break;
            }

            if rest_param {
                return Err(ParseError::unexpected(
                    cursor.next()?.expect("peeked token disappeared"),
                    "rest parameter must be the last formal parameter",
                ));
            }

            cursor.expect(Punctuator::Comma, "parameter list")?;
        }

        Ok(params.into_boxed_slice())
    }
}

/// Rest parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec]: https://tc39.es/ecma262/#prod-FunctionRestParameter
type FunctionRestParameter = BindingRestElement;

/// Rest parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec]: https://tc39.es/ecma262/#prod-BindingRestElement
#[derive(Debug, Clone, Copy)]
struct BindingRestElement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingRestElement {
    /// Creates a new `BindingRestElement` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for BindingRestElement
where
    R: Read,
{
    type Output = Vec<node::FormalParameter>;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("BindingRestElement", "Parsing");
        cursor.expect(Punctuator::Spread, "rest parameter")?;

        // TODO: BindingPattern
        

        let params_vec = if let Some(t) = cursor.peek(0)? {
            if *t.kind() == TokenKind::Punctuator(Punctuator::OpenBlock){
                std::println!("this is also happeing");// Delete this before submitting /////////////////////////////////////////////////////////////////////
                let mut params_list = Self::Output::new();
                let pattern_objects = ObjectBindingPattern::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

                let pattern_object_iter = pattern_objects.iter();
                for pattern_object in pattern_object_iter {
                    match pattern_object{
                        BindingPatternTypeObject::BindingPattern{ident, pattern: _, default_init} => {
                            let formal_parameter_object = node::FormalParameter::new(ident.clone(), default_init.clone(), true);
                        params_list.push(formal_parameter_object);
                        },
                        _ => {},
                    }     
                    
                }
                params_list
                
            }
            else{
                let mut params_list = Self::Output::new();
                let params = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
                let init = if let Some(t) = cursor.peek(0)? {
                    // Check that this is an initilizer before attempting parse.
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::new(true, self.allow_yield, self.allow_await).parse(cursor)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                params_list.push(node::FormalParameter::new(params, init, true));
                params_list
                
            }
        } else {
            Vec::new()
        };

        Ok(params_vec)
    }
}

/// Formal parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameter

#[derive(Debug, Clone, Copy)]
struct FormalParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameter {
    /// Creates a new `FormalParameter` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for FormalParameter
where
    R: Read,
{
    type Output = Declaration;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameter", "Parsing");

        // TODO: BindingPattern

        let params_vec = if let Some(t) = cursor.peek(0)? {
            if *t.kind() == TokenKind::Punctuator(Punctuator::OpenBlock)
            {
                std::println!("this is also happeing 3");// Delete this before submitting /////////////////////////////////////////////////////////////////////
                let mut params_list = Self::Output::new();
                let pattern_objects = ObjectBindingPattern::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

                std::println!("{:?}", pattern_objects);

                

                // let pattern_declaration = Declaration::new_with_object_pattern(pattern_objects.clone(), None);



            
                // params_list.push(node::FormalParameter::new(Box::new("").clone(), init.clone(), false));


                // let pattern_object_iter = pattern_objects.iter();
                // for pattern_object in pattern_object_iter {
                //     std::println!("{:?}", pattern_object);// Delete this before submitting /////////////////////////////////////////////////////////////////////
                //     match pattern_object{
                        
                //         BindingPatternTypeObject::SingleName{ident, property_name, default_init} => {
                //             std::println!("{:?}{}{:?}", ident, property_name, default_init);// Delete this before submitting /////////////////////////////////////////////////////////////////////
                //             let formal_parameter_object = node::FormalParameter::new(ident.clone(),  default_init.clone(), false);
                //         params_list.push(formal_parameter_object);
                //         },
                //         _ => {println!("this should not be happening")},// Delete this before submitting /////////////////////////////////////////////////////////////////////
                //     }       
                    
                // }
                params_list
            

            }
            else{
                let mut params_list = Self::Output::new();
                let params = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
                let init = if let Some(t) = cursor.peek(0)? {
                    // Check that this is an initilizer before attempting parse.
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::new(true, self.allow_yield, self.allow_await).parse(cursor)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                params_list.push(node::FormalParameter::new(params, init, false));
                params_list
                
            }
        } else {
            std::println!("this is also happeing 2");// Delete this before submitting /////////////////////////////////////////////////////////////////////
            Vec::new()
        };

     
        Ok(params_vec)
    }
}


/// A `FunctionBody` is equivalent to a `FunctionStatementList`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
pub(in crate::syntax::parser) type FunctionBody = FunctionStatementList;

/// The possible TokenKind which indicate the end of a function statement.
const FUNCTION_BREAK_TOKENS: [TokenKind; 1] = [TokenKind::Punctuator(Punctuator::CloseBlock)];

/// A function statement list
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionStatementList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FunctionStatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FunctionStatementList {
    /// Creates a new `FunctionStatementList` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for FunctionStatementList
where
    R: Read,
{
    type Output = node::StatementList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FunctionStatementList", "Parsing");

        let global_strict_mode = cursor.strict_mode();
        let mut strict = false;

        if let Some(tk) = cursor.peek(0)? {
            match tk.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    return Ok(Vec::new().into());
                }
                TokenKind::StringLiteral(string) if string.as_ref() == "use strict" => {
                    cursor.set_strict_mode(true);
                    strict = true;
                }
                _ => {}
            }
        }

        let statement_list = StatementList::new(
            self.allow_yield,
            self.allow_await,
            true,
            true,
            &FUNCTION_BREAK_TOKENS,
        )
        .parse(cursor);

        // Reset strict mode back to the global scope.
        cursor.set_strict_mode(global_strict_mode);

        let mut statement_list = statement_list?;
        statement_list.set_strict(strict);
        Ok(statement_list)
    }
}
