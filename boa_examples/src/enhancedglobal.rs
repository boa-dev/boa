use std::{fs::read_to_string};

use boa::{
    exec::Interpreter,
    forward,
    realm::Realm,
    builtins::value::Value,
    builtins::value::ResultValue,
    builtins::function::Function
};

pub fn run(){
    let js_file_path = "./scripts/enhancedglobal.js";
    let buffer = read_to_string(js_file_path);

    if buffer.is_err(){
        println!("Error: {}", buffer.unwrap_err());
        return;
    }

    //Creating the execution context
    let ctx = Realm::create();

    //Adding a custom global variable
    ctx.global_obj.set_field("customstring", "Hello! I am a custom global variable");

    //Adding a custom global function
    let rfn = Function::builtin(Vec::new(), rusty_hello);
    ctx.global_obj.set_field("rusty_hello", Value::from_func(rfn));

    //Adding s custom object
    let gobj = Value::new_object(Some(&ctx.global_obj));
    let addfn = Function::builtin(Vec::new(), add);
    gobj.set_field("add", Value::from_func(addfn));
    ctx.global_obj.set_field("rusty_obj", gobj);

    //Instantiating the engien with the execution context
    let mut engine = Interpreter::new(ctx);

    //Loading, parsing and executing the JS code from the source file
    let error_string = forward(&mut engine, &buffer.unwrap());
    if error_string != "undefined"{
        println!("Error parsing script: {}", error_string);
    }
}

//Custom function callable from JS
fn rusty_hello(_:&Value, args:&[Value], _:&mut Interpreter) -> ResultValue{
    let arg = args.get(0).unwrap();
    let val = format!("Hello from Rust! You passed {}", arg);
    return ResultValue::from(Ok(Value::from(val)));
}

//Function appended as property of a custom global object, callable from JS
fn add(_:&Value, args:&[Value], _engine:&mut Interpreter) -> ResultValue{
    let arg0 = args.get(0).unwrap();
    let arg1 = args.get(1).unwrap();
    return ResultValue::from(Ok(Value::from(arg0.to_integer() + arg1.to_integer())));
}