use boa_engine::{Context, Script, Source};

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn compile_bytecode(src: &'static str) -> String {
    let context = &mut Context::default();
    let source = Source::from_bytes(src);
    let script = Script::parse(source, None, context).unwrap();
    script.codeblock(context).unwrap().to_string()
}

macro_rules! test_case {
    ($fn_name:ident, $js:literal) => {
        #[test]
        fn $fn_name() {
            let output = compile_bytecode($js);
            insta::assert_snapshot!(output)
        }
    };
}

// Add test cases below
//
// Important note:
//
// The first arg is the function name / snapshot name
// The second arg is the js filename
//
test_case!(basic_loop, r"for (let i = 0; i < 100; ++i) {}");

test_case!(
    double_loop_function,
    r"
function f(x) {
  return x * x;
}

for (let n = 0; n < 20; n++) {
  for (let n = 0; n < 50; n++) {
    f(n);
  }
}

undefined;
"
);
