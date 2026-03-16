use boa_engine::{Context, Source};

fn main() {
    let mut context = Context::default();
    let res = context.eval(Source::from_bytes(r#"
        const wm = new WeakMap();
        try {
            wm.set(42, "value");
            false
        } catch (e) {
            e instanceof TypeError
        }
    "#)).unwrap();
    println!("{}", res.as_boolean().unwrap());
}
