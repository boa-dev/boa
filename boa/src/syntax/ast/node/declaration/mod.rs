//! Declaration nodes

pub mod arrow_function_decl;
pub mod const_decl_list;
pub mod function_decl;
pub mod function_expr;
pub mod let_decl_list;
pub mod var_decl_list;

pub use self::{
    arrow_function_decl::ArrowFunctionDecl,
    const_decl_list::{ConstDecl, ConstDeclList},
    function_decl::FunctionDecl,
    function_expr::FunctionExpr,
    let_decl_list::{LetDecl, LetDeclList},
    var_decl_list::{VarDecl, VarDeclList},
};
