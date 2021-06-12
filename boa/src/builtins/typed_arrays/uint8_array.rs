use crate::builtins::BuiltIn;
use crate::object::{ConstructorBuilder, PROTOTYPE};
use crate::property::{Attribute, DataDescriptor};
use crate::{Context, Result, Value};

#[derive(Debug, Clone, Copy)]
pub(crate) struct UInt8Array;

impl BuiltIn for UInt8Array {
    const NAME: &'static str = "Uint8Array";

    fn attribute() -> Attribute {
        // TODO: Figure out what this means because :shrug:
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let prototype = context.typed_array_prototype().clone().into();
        let constructor = ConstructorBuilder::new(context, Self::constructor)
            .inherit(prototype)
            .build();

        (Self::NAME, constructor.into(), Self::attribute())
    }
}

impl UInt8Array {
    fn constructor(new_target: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // Check if the argument is a number
        // -- instantiate a new array with that capacity
        // if it's an existing instance of a UInt8Array

        // -- return a value with a prototype of context.typed_array_prototype

        // TODO: Figure out this magical incantation -- from what i can tell this should result in the value
        // returned from TypedArray:init
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.typed_array_prototype().clone());

        let argument = args.get(0).expect("Expected an argument");
        match argument {
            Value::Integer(v) => {
                let typed_array = Value::new_object(context);
                typed_array
                    .as_object()
                    .expect("'Invariant. typed_array was created as an object'")
                    .set_prototype_instance(prototype.into());
                let length = DataDescriptor::new(
                    *v,
                    Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
                );
                typed_array.set_property("length", length);

                Ok(typed_array)
            }
            _ => {
                todo!()
            }
        }
    }
}
