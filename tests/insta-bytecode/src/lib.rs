#[test]
fn compile_bytecode() {
    use boa_engine::{Context, Script, Source};
    use insta::glob;

    glob!("../scripts/", "**/*.js", |path| {
        let context = &mut Context::default();
        let source = Source::from_filepath(path).expect("Could not load source");
        let script = Script::parse(source, None, context).unwrap();
        let output = script.codeblock(context).unwrap().to_string();
        insta::assert_snapshot!(output);
    });
}
