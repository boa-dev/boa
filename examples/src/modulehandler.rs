use std::fs::read_to_string;

use boa::{
    exec::Interpreter,
    forward,
    realm::Realm,
    builtins::value::Value,
    builtins::value::ResultValue,
    builtins::function::Function
};

pub fn run(){
    let js_file_path = "./scripts/calctest.js";
    let buffer = read_to_string(js_file_path);

    if buffer.is_err(){
        println!("Error: {}", buffer.unwrap_err());
        return;
    }

    //Creating the execution context
    let ctx = Realm::create();

    //Adding custom implementation that mimics 'require'
    let requirefn = Function::builtin(Vec::new(), require);
    ctx.global_obj.set_field("require", Value::from_func(requirefn));

    //Addming custom object that mimics 'module.exports'
    let moduleobj = Value::new_object(Some(&ctx.global_obj));
    moduleobj.set_field("exports", Value::from(" "));
    ctx.global_obj.set_field("module", moduleobj);

    //Instantiating the engien with the execution context
    let mut engine = Interpreter::new(ctx);

    //Loading, parsing and executing the JS code from the source file
    let error_string = forward(&mut engine, &buffer.unwrap());
    if error_string != "undefined"{
        println!("Error parsing script: {}", error_string);
    }
}

//Custom implementation that mimics 'require' module loader
fn require(_:&Value, args:&[Value], engine:&mut Interpreter) -> ResultValue{
    let arg = args.get(0).unwrap();
    
    //BUG: Dev branch seems to be passing string arguments along with quotes
    let libfile = arg.to_string().replace("\"", "");
    
    //Read the module source file
    println!("Loading: {}", libfile);
    let buffer = read_to_string(libfile);
    if buffer.is_err(){
        println!("Error: {}", buffer.unwrap_err());
        return ResultValue::from(Ok(Value::from(-1)));
    }else{
        //Load and parse the module source
        forward(engine, &buffer.unwrap());

        //Access module.exports and return as ResultValue
        let module_exports = engine.realm.global_obj.get_field("module").get_field("exports");
        let return_value = ResultValue::from(Ok(Value::from(module_exports)));
        return return_value;
    }
}