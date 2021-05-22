mod loadstring;
mod loadfile;
mod returnval;
mod enhancedglobal;
mod modulehandler;

fn main() {
    println!("\r\n");
    
    //example that loads, parses and executs a JS code string
    loadstring::run();
    println!("\r\n");

    //example that loads, parses and executs JS code from a source file (./scripts/helloworld.js)
    loadfile::run();
    println!("\r\n");

    //example that loads, parses and executs JS code and uses the return value
    returnval::run();
    println!("\r\n");

    //example that enhances the global object with custom values, objects, functions
    enhancedglobal::run();
    println!("\r\n");

    //example that implements a custom module handler which mimics (require / module.exports) pattern
    modulehandler::run();
    println!("\r\n");
}
