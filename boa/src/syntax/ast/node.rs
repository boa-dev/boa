use crate::syntax::ast::{
    constant::Const,
    op::{BinOp, Operator, UnaryOp},
};
use gc_derive::{Finalize, Trace};
use std::{
    collections::btree_map::BTreeMap,
    fmt::{Display, Formatter, Result},
};

#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A Javascript AST Node
pub enum Node {
    /// Create an array with items inside
    ArrayDecl(Vec<Node>),
    /// Create an arrow function with the given arguments and expression
    ArrowFunctionDecl(Vec<FormalParameter>, Box<Node>),
    /// Assign an expression to a value
    Assign(Box<Node>, Box<Node>),
    /// Run an operation between 2 expressions
    BinOp(BinOp, Box<Node>, Box<Node>),
    /// Run several expressions from top-to-bottom
    Block(Vec<Node>),
    /// Break statement with an optional label
    Break(Option<String>),
    /// Call a function with some values
    Call(Box<Node>, Vec<Node>),
    /// Conditional Operator ( ? : )
    ConditionalOp(Box<Node>, Box<Node>, Box<Node>),
    /// Make a constant value
    Const(Const),
    /// Const declaration
    ConstDecl(Vec<(String, Node)>),
    /// Construct an object from the function and arg{
    Construct(Box<Node>, Vec<Node>),
    /// Continue with an optional label
    Continue(Option<String>),
    /// Create a function with the given name, arguments, and expression
    FunctionDecl(Option<String>, Vec<FormalParameter>, Box<Node>),
    /// Gets the constant field of a value
    GetConstField(Box<Node>, String),
    /// Gets the [field] of a value
    GetField(Box<Node>, Box<Node>),
    /// [init], [cond], [step], body
    ForLoop(
        Option<Box<Node>>,
        Option<Box<Node>>,
        Option<Box<Node>>,
        Box<Node>,
    ),
    /// Check if a conditional expression is true and run an expression if it is and another expression if it isn't
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    /// Let declaraton
    LetDecl(Vec<(String, Option<Node>)>),
    /// Load a reference to a value, or a function argument
    Local(String),
    /// New
    New(Box<Node>),
    /// Create an object out of the binary tree given
    ObjectDecl(Box<BTreeMap<String, Node>>),
    /// Object Declaration (replaces ObjectDecl)
    Object(Vec<PropertyDefinition>),
    /// Return the expression from a function
    Return(Option<Box<Node>>),
    /// Run blocks whose cases match the expression
    Switch(Box<Node>, Vec<(Node, Vec<Node>)>, Option<Box<Node>>),
    /// Spread operator
    Spread(Box<Node>),
    // Similar to Block but without the braces
    StatementList(Vec<Node>),
    /// Throw a value
    Throw(Box<Node>),
    /// Return a string representing the type of the given expression
    TypeOf(Box<Node>),
    /// Try / Catch
    Try(
        Box<Node>,
        Option<Box<Node>>,
        Option<Box<Node>>,
        Option<Box<Node>>,
    ),
    This,
    /// Run an operation on a value
    UnaryOp(UnaryOp, Box<Node>),
    /// A variable declaration
    VarDecl(Vec<(String, Option<Node>)>),
    /// Repeatedly run an expression while the conditional expression resolves to true
    WhileLoop(Box<Node>, Box<Node>),
}

impl Operator for Node {
    fn get_assoc(&self) -> bool {
        match *self {
            Node::Construct(_, _)
            | Node::UnaryOp(_, _)
            | Node::TypeOf(_)
            | Node::If(_, _, _)
            | Node::Assign(_, _) => false,
            _ => true,
        }
    }
    fn get_precedence(&self) -> u64 {
        match self {
            Node::GetField(_, _) | Node::GetConstField(_, _) => 1,
            Node::Call(_, _) | Node::Construct(_, _) => 2,
            Node::UnaryOp(UnaryOp::IncrementPost, _)
            | Node::UnaryOp(UnaryOp::IncrementPre, _)
            | Node::UnaryOp(UnaryOp::DecrementPost, _)
            | Node::UnaryOp(UnaryOp::DecrementPre, _) => 3,
            Node::UnaryOp(UnaryOp::Not, _)
            | Node::UnaryOp(UnaryOp::Tilde, _)
            | Node::UnaryOp(UnaryOp::Minus, _)
            | Node::TypeOf(_) => 4,
            Node::BinOp(op, _, _) => op.get_precedence(),
            Node::If(_, _, _) => 15,
            // 16 should be yield
            Node::Assign(_, _) => 17,
            _ => 19,
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Node::Const(ref c) => write!(f, "{}", c),
            Node::ConditionalOp(_, _, _) => write!(f, "Conditional op"),
            Node::ForLoop(_, _, _, _) => write!(f, "for loop"),
            Node::This => write!(f, "this"),
            Node::Object(_) => write!(f, "object"),
            Node::Spread(_) => write!(f, "spread"),
            Node::New(_) => write!(f, "new"),
            Node::Try(_, _, _, _) => write!(f, "try/catch/finally"),
            Node::Break(_) => write!(f, "break"),
            Node::Continue(_) => write!(f, "continue"),
            Node::Block(ref block) => {
                write!(f, "{{")?;
                for expr in block.iter() {
                    write!(f, "{};", expr)?;
                }
                write!(f, "}}")
            }
            Node::StatementList(ref block) => {
                for expr in block.iter() {
                    write!(f, "{};", expr)?;
                }
                write!(f, "")
            }
            Node::Local(ref s) => write!(f, "{}", s),
            Node::GetConstField(ref ex, ref field) => write!(f, "{}.{}", ex, field),
            Node::GetField(ref ex, ref field) => write!(f, "{}[{}]", ex, field),
            Node::Call(ref ex, ref args) => {
                write!(f, "{}(", ex)?;
                let arg_strs: Vec<String> = args.iter().map(ToString::to_string).collect();
                write!(f, "{})", arg_strs.join(","))
            }
            Node::Construct(ref func, ref args) => {
                f.write_fmt(format_args!("new {}", func))?;
                f.write_str("(")?;
                let mut first = true;
                for e in args.iter() {
                    if !first {
                        f.write_str(", ")?;
                    }
                    first = false;
                    Display::fmt(e, f)?;
                }
                f.write_str(")")
            }
            Node::WhileLoop(ref cond, ref expr) => write!(f, "while({}) {}", cond, expr),
            Node::If(ref cond, ref expr, None) => write!(f, "if({}) {}", cond, expr),
            Node::If(ref cond, ref expr, Some(ref else_e)) => {
                write!(f, "if({}) {} else {}", cond, expr, else_e)
            }
            Node::Switch(ref val, ref vals, None) => {
                f.write_fmt(format_args!("switch({})", val))?;
                f.write_str(" {")?;
                for e in vals.iter() {
                    f.write_fmt(format_args!("case {}: \n", e.0))?;
                    join_expr(f, &e.1)?;
                }
                f.write_str("}")
            }
            Node::Switch(ref val, ref vals, Some(ref def)) => {
                f.write_fmt(format_args!("switch({})", val))?;
                f.write_str(" {")?;
                for e in vals.iter() {
                    f.write_fmt(format_args!("case {}: \n", e.0))?;
                    join_expr(f, &e.1)?;
                }
                f.write_str("default: \n")?;
                Display::fmt(def, f)?;
                f.write_str("}")
            }
            Node::ObjectDecl(ref map) => {
                f.write_str("{")?;
                for (key, value) in map.iter() {
                    f.write_fmt(format_args!("{}: {},", key, value))?;
                }
                f.write_str("}")
            }
            Node::ArrayDecl(ref arr) => {
                f.write_str("[")?;
                join_expr(f, arr)?;
                f.write_str("]")
            }
            Node::FunctionDecl(ref name, ref _args, ref expr) => {
                write!(f, "function ")?;
                if let Some(func_name) = name {
                    f.write_fmt(format_args!("{}", func_name))?;
                }
                write!(f, "{{")?;
                // join_expr(f, args)?;
                write!(f, "}} {}", expr)
            }
            Node::ArrowFunctionDecl(ref _args, ref expr) => {
                write!(f, "(")?;
                // join_expr(f, args.ini)?;
                write!(f, ") => {}", expr)
            }
            Node::BinOp(ref op, ref a, ref b) => write!(f, "{} {} {}", a, op, b),
            Node::UnaryOp(ref op, ref a) => write!(f, "{}{}", op, a),
            Node::Return(Some(ref ex)) => write!(f, "return {}", ex),
            Node::Return(None) => write!(f, "return"),
            Node::Throw(ref ex) => write!(f, "throw {}", ex),
            Node::Assign(ref ref_e, ref val) => write!(f, "{} = {}", ref_e, val),
            Node::VarDecl(ref vars) | Node::LetDecl(ref vars) => {
                if let Node::VarDecl(_) = *self {
                    f.write_str("var ")?;
                } else {
                    f.write_str("let ")?;
                }
                for (key, val) in vars.iter() {
                    match val {
                        Some(x) => f.write_fmt(format_args!("{} = {}", key, x))?,
                        None => f.write_fmt(format_args!("{}", key))?,
                    }
                }
                Ok(())
            }
            Node::ConstDecl(ref vars) => {
                f.write_str("const ")?;
                for (key, val) in vars.iter() {
                    f.write_fmt(format_args!("{} = {}", key, val))?
                }
                Ok(())
            }
            Node::TypeOf(ref e) => write!(f, "typeof {}", e),
        }
    }
}

/// `join_expr` - Utility to join multiple Expressions into a single string
fn join_expr(f: &mut Formatter, expr: &[Node]) -> Result {
    let mut first = true;
    for e in expr.iter() {
        if !first {
            f.write_str(", ")?;
        }
        first = false;
        Display::fmt(e, f)?;
    }
    Ok(())
}

// https://tc39.es/ecma262/#prod-FormalParameter
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct FormalParameter {
    pub name: String,
    pub init: Option<Box<Node>>,
    pub is_rest_param: bool,
}

pub type FormalParameters = Vec<FormalParameter>;

impl FormalParameter {
    pub fn new(name: String, init: Option<Box<Node>>, is_rest_param: bool) -> FormalParameter {
        FormalParameter {
            name: name,
            init: init,
            is_rest_param: is_rest_param,
        }
    }
}

// TODO: Support all features: https://tc39.github.io/ecma262/#prod-PropertyDefinition
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub enum PropertyDefinition {
    IdentifierReference(String),
    Property(String, Node),
    MethodDefinition(MethodDefinitionKind, String, Node),
    SpreadObject(Node),
}

#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub enum MethodDefinitionKind {
    Get,
    Set,
    Ordinary,
}
