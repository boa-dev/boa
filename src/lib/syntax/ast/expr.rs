use std::collections::btree_map::BTreeMap;
use std::fmt::{Display, Formatter, Result};
use syntax::ast::constant::Const;
use syntax::ast::op::{BinOp, Operator, UnaryOp};
use syntax::ast::pos::Position;

#[derive(Clone, PartialEq)]
pub struct Expr {
    /// The expression definition
    pub def: ExprDef,
    /// The starting position
    pub start: Position,
    /// The ending position
    pub end: Position,
}

impl Expr {
    /// Create a new expression with a starting and ending position
    pub fn new(def: ExprDef, start: Position, end: Position) -> Expr {
        Expr {
            def: def,
            start: start,
            end: end,
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.def)
    }
}

#[derive(Clone, PartialEq)]
/// A Javascript Expression
pub enum ExprDef {
    /// Run a operation between 2 expressions
    BinOpExpr(BinOp, Box<Expr>, Box<Expr>),
    /// Run an operation on a value
    UnaryOpExpr(UnaryOp, Box<Expr>),
    /// Make a constant value
    ConstExpr(Const),
    /// Construct an object from the function and arguments given
    ConstructExpr(Box<Expr>, Vec<Expr>),
    /// Run several expressions from top-to-bottom
    BlockExpr(Vec<Expr>),
    /// Load a reference to a value
    LocalExpr(String),
    /// Gets the constant field of a value
    GetConstFieldExpr(Box<Expr>, String),
    /// Gets the field of a value
    GetFieldExpr(Box<Expr>, Box<Expr>),
    /// Call a function with some values
    CallExpr(Box<Expr>, Vec<Expr>),
    /// Repeatedly run an expression while the conditional expression resolves to true
    WhileLoopExpr(Box<Expr>, Box<Expr>),
    /// Check if a conditional expression is true and run an expression if it is and another expression if it isn't
    IfExpr(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    /// Run blocks whose cases match the expression
    SwitchExpr(Box<Expr>, Vec<(Expr, Vec<Expr>)>, Option<Box<Expr>>),
    /// Create an object out of the binary tree given
    ObjectDeclExpr(Box<BTreeMap<String, Expr>>),
    /// Create an array with items inside
    ArrayDeclExpr(Vec<Expr>),
    /// Create a function with the given name, arguments, and expression
    FunctionDeclExpr(Option<String>, Vec<String>, Box<Expr>),
    /// Create an arrow function with the given arguments and expression
    ArrowFunctionDeclExpr(Vec<String>, Box<Expr>),
    /// Return the expression from a function
    ReturnExpr(Option<Box<Expr>>),
    /// Throw a value
    ThrowExpr(Box<Expr>),
    /// Assign an expression to a value
    AssignExpr(Box<Expr>, Box<Expr>),
    /// A variable declaration
    VarDeclExpr(Vec<(String, Option<Expr>)>),
    /// Return a string representing the type of the given expression
    TypeOfExpr(Box<Expr>),
}

impl Operator for ExprDef {
    fn get_assoc(&self) -> bool {
        match *self {
            ExprDef::ConstructExpr(_, _)
            | ExprDef::UnaryOpExpr(_, _)
            | ExprDef::TypeOfExpr(_)
            | ExprDef::IfExpr(_, _, _)
            | ExprDef::AssignExpr(_, _) => false,
            _ => true,
        }
    }
    fn get_precedence(&self) -> u64 {
        match *self {
            ExprDef::GetFieldExpr(_, _) | ExprDef::GetConstFieldExpr(_, _) => 1,
            ExprDef::CallExpr(_, _) | ExprDef::ConstructExpr(_, _) => 2,
            ExprDef::UnaryOpExpr(UnaryOp::IncrementPost, _)
            | ExprDef::UnaryOpExpr(UnaryOp::IncrementPre, _)
            | ExprDef::UnaryOpExpr(UnaryOp::DecrementPost, _)
            | ExprDef::UnaryOpExpr(UnaryOp::DecrementPre, _) => 3,
            ExprDef::UnaryOpExpr(UnaryOp::Not, _)
            | ExprDef::UnaryOpExpr(UnaryOp::Minus, _)
            | ExprDef::TypeOfExpr(_) => 4,
            ExprDef::BinOpExpr(op, _, _) => op.get_precedence(),
            ExprDef::IfExpr(_, _, _) => 15,
            // 16 should be yield
            ExprDef::AssignExpr(_, _) => 17,
            _ => 19,
        }
    }
}

impl Display for ExprDef {
    fn fmt(&self, f: &mut Formatter) -> Result {
        return match *self {
            ExprDef::ConstExpr(ref c) => write!(f, "{}", c),
            ExprDef::BlockExpr(ref block) => {
                try!(write!(f, "{}", "{"));
                for expr in block.iter() {
                    try!(write!(f, "{};", expr));
                }
                write!(f, "{}", "}")
            }
            ExprDef::LocalExpr(ref s) => write!(f, "{}", s),
            ExprDef::GetConstFieldExpr(ref ex, ref field) => write!(f, "{}.{}", ex, field),
            ExprDef::GetFieldExpr(ref ex, ref field) => write!(f, "{}[{}]", ex, field),
            ExprDef::CallExpr(ref ex, ref args) => {
                try!(write!(f, "{}(", ex));
                let arg_strs: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
                write!(f, "{})", arg_strs.connect(","))
            }
            ExprDef::ConstructExpr(ref func, ref args) => write!(f, "new {}({})", func, args),
            ExprDef::WhileLoopExpr(ref cond, ref expr) => write!(f, "while({}) {}", cond, expr),
            ExprDef::IfExpr(ref cond, ref expr, None) => write!(f, "if({}) {}", cond, expr),
            ExprDef::IfExpr(ref cond, ref expr, Some(ref else_e)) => {
                write!(f, "if({}) {} else {}", cond, expr, else_e)
            }
            ExprDef::SwitchExpr(ref val, ref vals, None) => write!(f, "switch({}){}", val, vals),
            ExprDef::SwitchExpr(ref val, ref vals, Some(ref def)) => {
                write!(f, "switch({}){}default:{}", val, vals, def)
            }
            ExprDef::ObjectDeclExpr(ref map) => write!(f, "{}", map),
            ExprDef::ArrayDeclExpr(ref arr) => write!(f, "{}", arr),
            ExprDef::FunctionDeclExpr(ref name, ref args, ref expr) => {
                write!(f, "function {}({}){}", name, args.connect(", "), expr)
            }
            ExprDef::ArrowFunctionDeclExpr(ref args, ref expr) => {
                write!(f, "({}) => {}", args.connect(", "), expr)
            }
            ExprDef::BinOpExpr(ref op, ref a, ref b) => write!(f, "{} {} {}", a, op, b),
            ExprDef::UnaryOpExpr(ref op, ref a) => write!(f, "{}{}", op, a),
            ExprDef::ReturnExpr(Some(ref ex)) => write!(f, "return {}", ex),
            ExprDef::ReturnExpr(None) => write!(f, "{}", "return"),
            ExprDef::ThrowExpr(ref ex) => write!(f, "throw {}", ex),
            ExprDef::AssignExpr(ref ref_e, ref val) => write!(f, "{} = {}", ref_e, val),
            ExprDef::VarDeclExpr(ref vars) => write!(f, "var {}", vars),
            ExprDef::TypeOfExpr(ref e) => write!(f, "typeof {}", e),
        };
    }
}
