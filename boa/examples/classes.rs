use boa::builtins::object::{Class, ClassBuilder};
use boa::builtins::value::*;
use boa::exec::*;
use boa::realm::Realm;
use boa::*;

use gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize)]
struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn say_hello(this: &Value, _: &[Value], ctx: &mut Interpreter) -> Result<Value> {
        if let Some(object) = this.as_object() {
            if let Some(person) = object.downcast_ref::<Person>() {
                println!(
                    "Hello my name is {}, I'm {} years old",
                    person.name, person.age
                );
                return Ok(Value::undefined());
            }
        }
        ctx.throw_type_error("'this' is not a Person object")
    }
}

impl Class for Person {
    const NAME: &'static str = "Person";
    const LENGTH: usize = 2;

    fn constructor(_this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Self> {
        let name = args.get(0).cloned().unwrap_or_default().to_string(ctx)?;
        let age = args.get(1).cloned().unwrap_or_default().to_u32(ctx)?;

        let person = Person {
            name: name.to_string(),
            age,
        };

        Ok(person)
    }

    fn methods(class: &mut ClassBuilder) -> Result<()> {
        class.method("sayHello", 0, Self::say_hello);
        class.static_method("is", 1, |_this, args, _ctx| {
            if let Some(arg) = args.get(0) {
                if let Some(object) = arg.as_object() {
                    if object.is::<Person>() {
                        return Ok(true.into());
                    }
                }
            }
            Ok(false.into())
        });

        Ok(())
    }
}

fn main() {
    let realm = Realm::create();
    let mut context = Interpreter::new(realm);

    context.register_global_class::<Person>().unwrap();

    forward_val(
        &mut context,
        r"
		let person = new Person('John', 19);
		person.sayHello();

		if (Person.is(person)) {
			console.log('person is a Person class instance.');
		}
		if (!Person.is('Hello')) {
			console.log('\'Hello\' string is not a Person class instance.');
		}
	",
    )
    .unwrap();
}
