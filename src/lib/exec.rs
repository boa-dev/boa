use ast::{
    CompEqual, CompGreaterThan, CompGreaterThanOrEqual, CompLessThan, CompLessThanOrEqual,
    CompNotEqual, CompStrictEqual, CompStrictNotEqual,
};
use ast::{LogAnd, LogOr};
use ast::{OpAdd, OpDiv, OpMod, OpMul, OpSub};
use ast::{UnaryMinus, UnaryNot};
use collections::treemap::TreeMap;
use js::function::{RegularFunc, RegularFunction};
use js::object::{ObjectData, INSTANCE_PROTOTYPE, PROTOTYPE};
use js::value::{
    from_value, to_value, ResultValue, VBoolean, VFunction, VInteger, VNull, VNumber, VObject,
    VString, VUndefined, Value, ValueData,
};
use js::{array, console, error, function, json, math, number, object, string, uri};
use std::cell::RefCell;
use std::gc::Gc;
use std::vec::Vec;
use syntax::ast::constant::Const;
use syntax::ast::expr::{Expr, ExprDef};
use syntax::ast::op::{BinOp, BitOp, CompOp, LogOp, NumOp, Operator, UnaryOp};
use syntax


/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new() -> Self;
    /// Set a global variable called `name` with the value `val`
    fn set_global(&mut self, name: StrBuf, val: Value) -> Value;
    /// Resolve the global variable `name`
    fn get_global(&self, name: StrBuf) -> Value;
    /// Create a new scope and return it
    fn make_scope(&mut self) -> Gc<RefCell<ObjectData>>;
    /// Destroy the current scope
    fn destroy_scope(&mut self) -> ();
    /// Run an expression
    fn run(&mut self, expr: &Expr) -> ResultValue;
}
#[deriving(Clone)]
/// A Javascript intepreter
pub struct Interpreter {
    /// An object representing the global object
    global: Value,
    /// The variable scopes
    scopes: Vec<Gc<RefCell<ObjectData>>>,
}
impl Executor for Interpreter {
    fn new() -> Interpreter {
        let global = ValueData::new_obj(None);
        object::init(global);
        console::init(global);
        math::init(global);
        array::init(global);
        function::init(global);
        json::init(global);
        number::init(global);
        error::init(global);
        string::init(global);
        uri::init(global);
        Interpreter {
            global: global,
            scopes: Vec::new(),
        }
    }
    fn set_global(&mut self, name: StrBuf, val: Value) -> Value {
        self.global.borrow().set_field(name, val)
    }
    fn get_global(&self, name: StrBuf) -> Value {
        self.global.borrow().get_field(name)
    }
    fn make_scope(&mut self) -> Gc<RefCell<ObjectData>> {
        let value = Gc::new(RefCell::new(TreeMap::new()));
        self.scopes.push(value.clone());
        value
    }
    fn destroy_scope(&mut self) -> () {
        self.scopes.pop();
    }
    fn run(&mut self, expr: &Expr) -> ResultValue {
        match expr.def {
            ConstExpr(CNull) => Ok(Gc::new(VNull)),
            ConstExpr(CUndefined) => Ok(Gc::new(VUndefined)),
            ConstExpr(CNum(num)) => Ok(to_value(num)),
            ConstExpr(CInt(num)) => Ok(to_value(num)),
            ConstExpr(CString(ref str)) => Ok(Gc::new(VString(StrBuf::from_str(str.as_slice())))),
            ConstExpr(CBool(val)) => Ok(Gc::new(VBoolean(val))),
            ConstExpr(CRegExp(_, _, _)) => Ok(Gc::new(VNull)),
            BlockExpr(ref es) => {
                let mut obj = Gc::new(VNull);
                for e in es.iter() {
                    let val = try!(self.run(e));
                    if e == es.last().unwrap() {
                        obj = val;
                    }
                }
                Ok(obj)
            }
            LocalExpr(ref name) => {
                let mut value = Gc::new(VUndefined);
                for scope in self.scopes.iter().rev() {
                    match scope.borrow().borrow().find(name) {
                        Some(v) => {
                            value = v.value.clone();
                            break;
                        }
                        None => (),
                    }
                }
                Ok(if value.borrow().is_undefined() {
                    self.global.borrow().get_field(name.clone())
                } else {
                    value
                })
            }
            GetConstFieldExpr(ref obj, ref field) => {
                let val_obj = try!(self.run(*obj));
                Ok(val_obj.borrow().get_field(field.clone()))
            }
            GetFieldExpr(ref obj, ref field) => {
                let val_obj = try!(self.run(*obj));
                let val_field = try!(self.run(*field));
                Ok(val_obj.borrow().get_field(val_field.borrow().to_str()))
            }
            CallExpr(ref callee, ref args) => {
                let (this, func) = match callee.def {
                    GetConstFieldExpr(ref obj, ref field) => {
                        let obj = try!(self.run(*obj));
                        (obj, obj.borrow().get_field(field.clone()))
                    }
                    GetFieldExpr(ref obj, ref field) => {
                        let obj = try!(self.run(*obj));
                        let field = try!(self.run(*field));
                        (obj, obj.borrow().get_field(field.borrow().to_str()))
                    }
                    _ => (self.global.clone(), try!(self.run(callee.clone()))),
                };
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(try!(self.run(arg)));
                }
                match *func.borrow() {
                    VFunction(ref func) => func.borrow().call(self, this, Gc::new(VNull), v_args),
                    _ => Err(Gc::new(VUndefined)),
                }
            }
            WhileLoopExpr(ref cond, ref expr) => {
                let mut result = Gc::new(VUndefined);
                while try!(self.run(*cond)).borrow().is_true() {
                    result = try!(self.run(*expr));
                }
                Ok(result)
            }
            IfExpr(ref cond, ref expr, None) => Ok(if try!(self.run(*cond)).borrow().is_true() {
                try!(self.run(*expr))
            } else {
                Gc::new(VUndefined)
            }),
            IfExpr(ref cond, ref expr, Some(ref else_e)) => {
                Ok(if try!(self.run(*cond)).borrow().is_true() {
                    try!(self.run(*expr))
                } else {
                    try!(self.run(*else_e))
                })
            }
            SwitchExpr(ref val_e, ref vals, ref default) => {
                let val = try!(self.run(*val_e)).borrow().clone();
                let mut result = Gc::new(VNull);
                let mut matched = false;
                for tup in vals.iter() {
                    let tup: &(Expr, Vec<Expr>) = tup;
                    match *tup {
                        (ref cond, ref block) if (val == *try!(self.run(cond)).borrow()) => {
                            matched = true;
                            let last_expr = block.last().unwrap();
                            for expr in block.iter() {
                                let e_result = try!(self.run(expr));
                                if expr == last_expr {
                                    result = e_result;
                                }
                            }
                        }
                        _ => (),
                    }
                }
                if !matched && default.is_some() {
                    result = try!(self.run(*default.as_ref().unwrap()));
                }
                Ok(result)
            }
            ObjectDeclExpr(ref map) => {
                let obj = ValueData::new_obj(Some(self.global));
                for (key, val) in map.iter() {
                    obj.borrow().set_field(key.clone(), try!(self.run(val)));
                }
                Ok(obj)
            }
            ArrayDeclExpr(ref arr) => {
                let arr_map = ValueData::new_obj(Some(self.global));
                let mut index: i32 = 0;
                for val in arr.iter() {
                    let val = try!(self.run(val));
                    arr_map.borrow().set_field(index.to_str(), val);
                    index += 1;
                }
                arr_map.borrow().set_field_slice(
                    INSTANCE_PROTOTYPE,
                    self.get_global("Array".into_strbuf())
                        .borrow()
                        .get_field_slice(PROTOTYPE),
                );
                arr_map.borrow().set_field_slice("length", to_value(index));
                Ok(arr_map)
            }
            FunctionDeclExpr(ref name, ref args, ref expr) => {
                let function = RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                let val = Gc::new(VFunction(RefCell::new(function)));
                if name.is_some() {
                    self.global.borrow().set_field(name.clone().unwrap(), val);
                }
                Ok(val)
            }
            ArrowFunctionDeclExpr(ref args, ref expr) => {
                let function = RegularFunc(RegularFunction::new(*expr.clone(), args.clone()));
                Ok(Gc::new(VFunction(RefCell::new(function))))
            }
            BinOpExpr(BinNum(ref op), ref a, ref b) => {
                let v_r_a = try!(self.run(*a));
                let v_r_b = try!(self.run(*b));
                let v_a = v_r_a.borrow();
                let v_b = v_r_b.borrow();
                Ok(Gc::new(match *op {
                    OpAdd => *v_a + *v_b,
                    OpSub => *v_a - *v_b,
                    OpMul => *v_a * *v_b,
                    OpDiv => *v_a / *v_b,
                    OpMod => *v_a % *v_b,
                }))
            }
            UnaryOpExpr(ref op, ref a) => {
                let v_r_a = try!(self.run(*a));
                let v_a = v_r_a.borrow();
                Ok(match *op {
                    UnaryMinus => to_value(-v_a.to_num()),
                    UnaryNot => Gc::new(!v_a),
                    _ => unreachable!(),
                })
            }
            BinOpExpr(BinBit(ref op), ref a, ref b) => {
                let v_r_a = try!(self.run(*a));
                let v_r_b = try!(self.run(*b));
                let v_a = v_r_a.borrow();
                let v_b = v_r_b.borrow();
                Ok(Gc::new(match *op {
                    BitAnd => *v_a & *v_b,
                    BitOr => *v_a | *v_b,
                    BitXor => *v_a ^ *v_b,
                    BitShl => *v_a << *v_b,
                    BitShr => *v_a >> *v_b,
                }))
            }
            BinOpExpr(BinComp(ref op), ref a, ref b) => {
                let v_r_a = try!(self.run(*a));
                let v_r_b = try!(self.run(*b));
                let v_a = v_r_a.borrow();
                let v_b = v_r_b.borrow();
                Ok(to_value(match *op {
                    CompEqual if v_a.is_object() => v_r_a.ptr_eq(&v_r_b),
                    CompEqual => v_a == v_b,
                    CompNotEqual if v_a.is_object() => !v_r_a.ptr_eq(&v_r_b),
                    CompNotEqual => v_a != v_b,
                    CompStrictEqual if v_a.is_object() => v_r_a.ptr_eq(&v_r_b),
                    CompStrictEqual => v_a == v_b,
                    CompStrictNotEqual if v_a.is_object() => !v_r_a.ptr_eq(&v_r_b),
                    CompStrictNotEqual => v_a != v_b,
                    CompGreaterThan => v_a.to_num() > v_b.to_num(),
                    CompGreaterThanOrEqual => v_a.to_num() >= v_b.to_num(),
                    CompLessThan => v_a.to_num() < v_b.to_num(),
                    CompLessThanOrEqual => v_a.to_num() <= v_b.to_num(),
                }))
            }
            BinOpExpr(BinLog(ref op), ref a, ref b) => {
                let v_a = from_value::<bool>(try!(self.run(*a))).unwrap();
                let v_b = from_value::<bool>(try!(self.run(*b))).unwrap();
                Ok(match *op {
                    LogAnd => to_value(v_a && v_b),
                    LogOr => to_value(v_a || v_b),
                })
            }
            ConstructExpr(ref callee, ref args) => {
                let func = try!(self.run(*callee));
                let mut v_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    v_args.push(try!(self.run(arg)));
                }
                let this = Gc::new(VObject(RefCell::new(TreeMap::new())));
                this.borrow()
                    .set_field_slice(INSTANCE_PROTOTYPE, func.borrow().get_field_slice(PROTOTYPE));
                Ok(match *func.borrow() {
                    VFunction(ref func) => {
                        try!(func.borrow().call(self, this, Gc::new(VNull), v_args));
                        this
                    }
                    _ => Gc::new(VUndefined),
                })
            }
            ReturnExpr(ref ret) => match *ret {
                Some(ref v) => self.run(*v),
                None => Ok(Gc::new(VUndefined)),
            },
            ThrowExpr(ref ex) => Err(try!(self.run(*ex))),
            AssignExpr(ref ref_e, ref val_e) => {
                let val = try!(self.run(*val_e));
                match ref_e.def {
                    LocalExpr(ref name) => {
                        self.global.borrow().set_field(name.clone(), val);
                    }
                    GetConstFieldExpr(ref obj, ref field) => {
                        let val_obj = try!(self.run(*obj));
                        val_obj.borrow().set_field(field.clone(), val);
                    }
                    _ => (),
                }
                Ok(val)
            }
            TypeOfExpr(ref val_e) => {
                let val = try!(self.run(*val_e));
                Ok(to_value(match *val.borrow() {
                    VUndefined => "undefined",
                    VNull | VObject(_) => "object",
                    VBoolean(_) => "boolean",
                    VNumber(_) | VInteger(_) => "number",
                    VString(_) => "string",
                    VFunction(_) => "function",
                }))
            }
        }
    }
}
