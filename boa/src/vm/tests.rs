use super::{compilation::*, *};
use crate::parser_expr;

#[test]
fn poc() {
    let src = r#"
7 + 4.1 + 1;
"#;

    let l = parser_expr(src).expect("parsing failed");
    let instrs = Compiler::new().compile(&l);

    let mut vm = VM::new(Realm::create());

    let res = vm.run(&instrs).unwrap();

    assert_eq!(12.1, res.data().to_number());
}
