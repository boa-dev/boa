use std::{fs::read_to_string};

use boa::{
    exec::Interpreter,
    forward,
    realm::Realm
};

pub fn run(){
    let js_file_path = "./scripts/helloworld.js";
    let buffer = read_to_string(js_file_path);

    if buffer.is_err(){
        println!("Error: {}", buffer.unwrap_err());
        return;
    }

    //Create the execution context
    let ctx = Realm::create();

    //Instantiate the engien with the execution context
    let mut engine = Interpreter::new(ctx);

    //Load, parse and execute the JS code read from the source file
    let error_string = forward(&mut engine, &buffer.unwrap());
    if error_string != "undefined"{
        println!("Error parsing script: {}", error_string);
    }
}