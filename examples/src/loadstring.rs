use boa::{
    exec::Interpreter,
    forward,
    realm::Realm
};

pub fn run(){
    let js_code = "console.log('Hello World from a JS code string!')";

    //Create the execution context
    let ctx = Realm::create();

    //Instantiate the engien with the execution context
    let mut engine = Interpreter::new(ctx);

    //Load, parse and execute the given JS String
    let error_string = forward(&mut engine, js_code);
    if error_string != "undefined"{
        println!("Error parsing script: {}", error_string);
    }
}