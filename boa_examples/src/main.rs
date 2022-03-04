mod classes;
mod closures;
mod jsarray;
mod loadfile;
mod loadstring;
mod modulehandler;

fn main() {
    println!("\r\n");

    //example that loads, parses and executs a JS code string
    loadstring::run();
    println!("\r\n");

    //example that loads, parses and executs JS code from a source file (./scripts/helloworld.js)
    loadfile::run();
    println!("\r\n");

    //example that implements classes in Rust and exposes them to JS
    classes::run();
    println!("\r\n");

    //example that implements closures in Rust and exposes them to JS
    closures::run().expect("closures failed to excecute");
    println!("\r\n");

    //example that shows array manipulation in Rust
    jsarray::run().expect("closures failed to excecute");
    println!("\r\n");

    //example that implements a custom module handler which mimics (require / module.exports) pattern
    modulehandler::run();
    println!("\r\n");
}
