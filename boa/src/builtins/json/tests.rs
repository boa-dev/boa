use crate::{exec::Executor, forward, realm::Realm};

#[test]
fn json_sanity() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    assert_eq!(
        forward(&mut engine, r#"JSON.parse('{"aaa":"bbb"}').aaa == 'bbb'"#),
        "true"
    );
    assert_eq!(
        forward(
            &mut engine,
            r#"JSON.stringify({aaa: 'bbb'}) == '{"aaa":"bbb"}'"#
        ),
        "true"
    );
}
