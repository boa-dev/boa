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
    builtins::BuiltIn,
    object::{ConstructorBuilder, Object as BuiltinObject, ObjectData},
    property::{Attribute, Property},
    value::{same_value, Value},
    BoaProfiler, Context, Result,
};

#[cfg(test)]
mod tests;

/// The global JavaScript object.
#[derive(Debug, Clone, Copy)]
pub struct Object;

impl BuiltIn for Object {
    const NAME: &'static str = "Object";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().object_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(Value::null())
        .method(Self::has_own_property, "hasOwnProperty", 0)
        .method(Self::property_is_enumerable, "propertyIsEnumerable", 0)
        .method(Self::to_string, "toString", 0)
        .static_method(Self::create, "create", 2)
        .static_method(Self::set_prototype_of, "setPrototypeOf", 2)
        .static_method(Self::get_prototype_of, "getPrototypeOf", 1)
        .static_method(Self::define_property, "defineProperty", 3)
        .static_method(Self::is, "is", 2)
        .build();

        (Self::NAME, object.into(), Self::attribute())
    }
}

impl Object {
    const LENGTH: usize = 1;

    fn constructor(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if let Some(arg) = args.get(0) {
            if !arg.is_null_or_undefined() {
                return Ok(arg.to_object(context)?.into());
            }
        }
        let global = context.global_object();

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
            Value::Object(_) | Value::Null => Ok(Value::object(BuiltinObject::with_prototype(
                prototype,
                ObjectData::Ordinary,
            ))),
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
    pub fn to_string(this: &Value, _: &[Value], ctx: &mut Context) -> Result<Value> {
        if this.is_undefined() {
            Ok("[object Undefined]".into())
        } else if this.is_null() {
            Ok("[object Null]".into())
        } else {
            let gc_o = this.to_object(ctx)?;
            let o = gc_o.borrow();
            let builtin_tag = match &o.data {
                ObjectData::Array => "Array",
                // TODO: Arguments Exotic Objects are currently not supported
                ObjectData::Function(_) => "Function",
                ObjectData::Error => "Error",
                ObjectData::Boolean(_) => "Boolean",
                ObjectData::Number(_) => "Number",
                ObjectData::String(_) => "String",
                ObjectData::Date(_) => "Date",
                ObjectData::RegExp(_) => "RegExp",
                _ => "Object",
            };

            let tag = o.get(&ctx.well_known_symbols().to_string_tag_symbol().into());

            let tag_str = tag.as_string().map(|s| s.as_str()).unwrap_or(builtin_tag);

            Ok(format!("[object {}]", tag_str).into())
        }
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
        let own_property = this
            .to_object(ctx)
            .map(|obj| obj.borrow().get_own_property(&key));

        Ok(own_property.map_or(Value::from(false), |own_prop| {
            Value::from(own_prop.enumerable_or(false))
        }))
    }
}
