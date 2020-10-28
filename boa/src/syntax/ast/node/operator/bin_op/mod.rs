use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::{
        node::Node,
        op::{self, AssignOp, BitOp, CompOp, LogOp, NumOp},
    },
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Binary operators requires two operands, one before the operator and one after the operator.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct BinOp {
    op: op::BinOp,
    lhs: Box<Node>,
    rhs: Box<Node>,
}

impl BinOp {
    /// Creates a `BinOp` AST node.
    pub(in crate::syntax) fn new<O, L, R>(op: O, lhs: L, rhs: R) -> Self
    where
        O: Into<op::BinOp>,
        L: Into<Node>,
        R: Into<Node>,
    {
        Self {
            op: op.into(),
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
        }
    }

    /// Gets the binary operation of the node.
    pub fn op(&self) -> op::BinOp {
        self.op
    }

    /// Gets the left hand side of the binary operation.
    pub fn lhs(&self) -> &Node {
        &self.lhs
    }

    /// Gets the right hand side of the binary operation.
    pub fn rhs(&self) -> &Node {
        &self.rhs
    }

    /// Runs the assignment operators.
    fn run_assign(op: AssignOp, x: Value, y: Value, context: &mut Context) -> Result<Value> {
        match op {
            AssignOp::Add => x.add(&y, context),
            AssignOp::Sub => x.sub(&y, context),
            AssignOp::Mul => x.mul(&y, context),
            AssignOp::Exp => x.pow(&y, context),
            AssignOp::Div => x.div(&y, context),
            AssignOp::Mod => x.rem(&y, context),
            AssignOp::And => x.bitand(&y, context),
            AssignOp::Or => x.bitor(&y, context),
            AssignOp::Xor => x.bitxor(&y, context),
            AssignOp::Shl => x.shl(&y, context),
            AssignOp::Shr => x.shr(&y, context),
            AssignOp::Ushr => x.ushr(&y, context),
        }
    }
}

impl Executable for BinOp {
    fn run(&self, context: &mut Context) -> Result<Value> {
        match self.op() {
            op::BinOp::Num(op) => {
                let x = self.lhs().run(context)?;
                let y = self.rhs().run(context)?;
                match op {
                    NumOp::Add => x.add(&y, context),
                    NumOp::Sub => x.sub(&y, context),
                    NumOp::Mul => x.mul(&y, context),
                    NumOp::Exp => x.pow(&y, context),
                    NumOp::Div => x.div(&y, context),
                    NumOp::Mod => x.rem(&y, context),
                }
            }
            op::BinOp::Bit(op) => {
                let x = self.lhs().run(context)?;
                let y = self.rhs().run(context)?;
                match op {
                    BitOp::And => x.bitand(&y, context),
                    BitOp::Or => x.bitor(&y, context),
                    BitOp::Xor => x.bitxor(&y, context),
                    BitOp::Shl => x.shl(&y, context),
                    BitOp::Shr => x.shr(&y, context),
                    BitOp::UShr => x.ushr(&y, context),
                }
            }
            op::BinOp::Comp(op) => {
                let x = self.lhs().run(context)?;
                let y = self.rhs().run(context)?;
                Ok(Value::from(match op {
                    CompOp::Equal => x.equals(&y, context)?,
                    CompOp::NotEqual => !x.equals(&y, context)?,
                    CompOp::StrictEqual => x.strict_equals(&y),
                    CompOp::StrictNotEqual => !x.strict_equals(&y),
                    CompOp::GreaterThan => x.gt(&y, context)?,
                    CompOp::GreaterThanOrEqual => x.ge(&y, context)?,
                    CompOp::LessThan => x.lt(&y, context)?,
                    CompOp::LessThanOrEqual => x.le(&y, context)?,
                    CompOp::In => {
                        if !y.is_object() {
                            return context.throw_type_error(format!(
                                "right-hand side of 'in' should be an object, got {}",
                                y.get_type().as_str()
                            ));
                        }
                        let key = x.to_property_key(context)?;
                        context.has_property(&y, &key)
                    }
                    CompOp::InstanceOf => {
                        if let Some(object) = y.as_object() {
                            let key = context.well_known_symbols().has_instance_symbol();

                            match object.get_method(context, key)? {
                                Some(instance_of_handler) => {
                                    instance_of_handler.call(&y, &[x], context)?.to_boolean()
                                }
                                None if object.is_callable() => {
                                    object.ordinary_has_instance(context, &x)?
                                }
                                None => {
                                    return context.throw_type_error(
                                        "right-hand side of 'instanceof' is not callable",
                                    );
                                }
                            }
                        } else {
                            return context.throw_type_error(format!(
                                "right-hand side of 'instanceof' should be an object, got {}",
                                y.get_type().as_str()
                            ));
                        }
                    }
                }))
            }
            op::BinOp::Log(op) => {
                // turn a `Value` into a `bool`
                let to_bool = |value| bool::from(&value);
                Ok(match op {
                    LogOp::And => Value::from(
                        to_bool(self.lhs().run(context)?) && to_bool(self.rhs().run(context)?),
                    ),
                    LogOp::Or => Value::from(
                        to_bool(self.lhs().run(context)?) || to_bool(self.rhs().run(context)?),
                    ),
                })
            }
            op::BinOp::Assign(op) => match self.lhs() {
                Node::Identifier(ref name) => {
                    let v_a = context
                        .realm()
                        .environment
                        .get_binding_value(name.as_ref())
                        .ok_or_else(|| context.construct_reference_error(name.as_ref()))?;
                    let v_b = self.rhs().run(context)?;
                    let value = Self::run_assign(op, v_a, v_b, context)?;
                    context.realm_mut().environment.set_mutable_binding(
                        name.as_ref(),
                        value.clone(),
                        true,
                    );
                    Ok(value)
                }
                Node::GetConstField(ref get_const_field) => {
                    let v_r_a = get_const_field.obj().run(context)?;
                    let v_a = v_r_a.get_field(get_const_field.field());
                    let v_b = self.rhs().run(context)?;
                    let value = Self::run_assign(op, v_a, v_b, context)?;
                    v_r_a.set_field(get_const_field.field(), value.clone());
                    Ok(value)
                }
                _ => Ok(Value::undefined()),
            },
            op::BinOp::Comma => {
                self.lhs().run(context)?;
                Ok(self.rhs().run(context)?)
            }
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}

impl From<BinOp> for Node {
    fn from(op: BinOp) -> Self {
        Self::BinOp(op)
    }
}
