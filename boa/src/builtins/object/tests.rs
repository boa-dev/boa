use crate::{exec::Interpreter, forward, realm::Realm};

#[test]
fn object_has_own_property() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        let x = { someProp: 1, undefinedProp: undefined, nullProp: null };
    "#;

    eprintln!("{}", forward(&mut engine, init));
    assert_eq!(forward(&mut engine, "x.hasOwnProperty('someProp')"), "true");
    assert_eq!(
        forward(&mut engine, "x.hasOwnProperty('undefinedProp')"),
        "true"
    );
    assert_eq!(forward(&mut engine, "x.hasOwnProperty('nullProp')"), "true");
    assert_eq!(
        forward(&mut engine, "x.hasOwnProperty('hasOwnProperty')"),
        "false"
    );
}
