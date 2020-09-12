//! This module implements the global `Object` object.
//!
//! The `Object` class represents one of JavaScript's data types.
//!
//! It is used to store various keyed collections and more complex entities.
//! Objects can be created using the `Object()` constructor or the
//! object initializer / literal syntax.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object

use crate::{
    builtins::function::{make_builtin_fn, make_constructor_fn},
    object::ObjectData,
    property::Property,
    value::{same_value, Value},
    BoaProfiler, Context, Result,
};

#[cfg(test)]
mod tests;

/// The global JavaScript object.
#[derive(Debug, Clone, Copy)]
pub struct Object;

impl Object {
    /// Create a new object.
    pub fn make_object(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        if let Some(arg) = args.get(0) {
            if !arg.is_null_or_undefined() {
                return arg.to_object(ctx);
            }
        }
        let global = ctx.global_object();

        Ok(Value::new_object(Some(global)))
    }

    /// `Object.create( proto, [propertiesObject] )`
    ///
    /// Creates a new object from the provided prototype.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.create
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/create
    pub fn create(_: &Value, args: &[Value], interpreter: &mut Context) -> Result<Value> {
        let prototype = args.get(0).cloned().unwrap_or_else(Value::undefined);
        let properties = args.get(1).cloned().unwrap_or_else(Value::undefined);

        if properties != Value::Undefined {
            unimplemented!("propertiesObject argument of Object.create")
        }

        match prototype {
            Value::Object(_) | Value::Null => Ok(Value::new_object_from_prototype(
                prototype,
                ObjectData::Ordinary,
            )),
            _ => interpreter.throw_type_error(format!(
                "Object prototype may only be an Object or null: {}",
                prototype.display()
            )),
        }
    }

    /// Uses the SameValue algorithm to check equality of objects
    pub fn is(_: &Value, args: &[Value], _: &mut Context) -> Result<Value> {
        let x = args.get(0).cloned().unwrap_or_else(Value::undefined);
        let y = args.get(1).cloned().unwrap_or_else(Value::undefined);

        Ok(same_value(&x, &y).into())
    }

    /// Get the `prototype` of an object.
    pub fn get_prototype_of(_: &Value, args: &[Value], _: &mut Context) -> Result<Value> {
        let obj = args.get(0).expect("Cannot get object");
        Ok(obj.as_object().map_or_else(Value::undefined, |object| {
            object.prototype_instance().clone()
        }))
    }

    /// Set the `prototype` of an object.
    pub fn set_prototype_of(_: &Value, args: &[Value], _: &mut Context) -> Result<Value> {
        let obj = args.get(0).expect("Cannot get object").clone();
        let proto = args.get(1).expect("Cannot get object").clone();
        obj.as_object_mut().unwrap().set_prototype_instance(proto);
        Ok(obj)
    }

    /// Define a property in an object
    pub fn define_property(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let obj = args.get(0).expect("Cannot get object");
        let prop = args.get(1).expect("Cannot get object").to_string(ctx)?;
        let desc = Property::from(args.get(2).expect("Cannot get object"));
        obj.set_property(prop, desc);
        Ok(Value::undefined())
    }

    /// `Object.prototype.toString()`
    ///
    /// This method returns a string representing the object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toString
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
        // FIXME: it should not display the object.
        Ok(this.display().to_string().into())
    }

    /// `Object.prototype.hasOwnPrototype( property )`
    ///
    /// The method returns a boolean indicating whether the object has the specified property
    /// as its own property (as opposed to inheriting it).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.prototype.hasownproperty
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwnProperty
    pub fn has_own_property(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let prop = if args.is_empty() {
            None
        } else {
            Some(args.get(0).expect("Cannot get object").to_string(ctx)?)
        };
        let own_property = this
            .as_object()
            .as_deref()
            .expect("Cannot get THIS object")
            .get_own_property(&prop.expect("cannot get prop").into());
        if own_property.is_none() {
            Ok(Value::from(false))
        } else {
            Ok(Value::from(true))
        }
    }

    pub fn property_is_enumerable(
        this: &Value,
        args: &[Value],
        ctx: &mut Context,
    ) -> Result<Value> {
        let key = match args.get(0) {
            None => return Ok(Value::from(false)),
            Some(key) => key,
        };

        let key = key.to_property_key(ctx)?;
        let own_property = this.to_object(ctx).map(|obj| {
            obj.as_object()
                .expect("Unable to deref object")
                .get_own_property(&key)
        });

        Ok(own_property.map_or(Value::from(false), |own_prop| {
            Value::from(own_prop.enumerable_or(false))
        }))
    }

    /// Initialise the `Object` object on the global object.
    #[inline]
    pub fn init(interpreter: &mut Context) -> (&'static str, Value) {
        let global = interpreter.global_object();
        let _timer = BoaProfiler::global().start_event("object", "init");

        let prototype = Value::new_object(None);

        make_builtin_fn(
            Self::has_own_property,
            "hasOwnProperty",
            &prototype,
            0,
            interpreter,
        );
        make_builtin_fn(
            Self::property_is_enumerable,
            "propertyIsEnumerable",
            &prototype,
            0,
            interpreter,
        );
        make_builtin_fn(Self::to_string, "toString", &prototype, 0, interpreter);

        let object = make_constructor_fn(
            "Object",
            1,
            Self::make_object,
            global,
            prototype,
            true,
            true,
        );

        // static methods of the builtin Object
        make_builtin_fn(Self::create, "create", &object, 2, interpreter);
        make_builtin_fn(
            Self::set_prototype_of,
            "setPrototypeOf",
            &object,
            2,
            interpreter,
        );
        make_builtin_fn(
            Self::get_prototype_of,
            "getPrototypeOf",
            &object,
            1,
            interpreter,
        );
        make_builtin_fn(
            Self::define_property,
            "defineProperty",
            &object,
            3,
            interpreter,
        );
        make_builtin_fn(Self::is, "is", &object, 2, interpreter);

        ("Object", object)
    }
}
