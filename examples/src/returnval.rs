use boa::{
    exec::Interpreter,
    forward_val,
    realm::Realm
};

pub fn run(){
    let js_string = r"
        function add(a, b){
            return a + b;
        }
        let a = 3;
        let b = 3;
        add(a,b);
    ";

    //Create the execution context
    let ctx = Realm::create();

    //Instantiate the engien with the execution context
    let mut engine = Interpreter::new(ctx);

    //Load, parse, execute the given JS String and returns the result of the execution as a Boa Value
    match forward_val(&mut engine, js_string){
        Ok(v) => {
            println!("Script returned: {}", v);
        },
        Err(e) =>{
            println!("Script error: {}", e);
        }
    }
}