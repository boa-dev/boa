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

#[test]
fn json_parse_array_with_reviver() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let result = forward(
        &mut engine,
        r#"JSON.parse('[1,2,3,4]', function(k, v){
            if (typeof v == 'number') {
                return v * 2;
            } else {
                v
        }})"#,
    );
    assert_eq!(
        result,
        "{\n    3: 8,\n    2: 6,\n    0: 2,\n    1: 4,\n    length: 4,\n    extensible: true\n}"
    );
}

#[test]
fn json_parse_object_with_reviver() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let result = forward(
        &mut engine,
        r#"
        var myObj = new Object();
        myObj.firstname = "boa";
        myObj.lastname = "snake";
        var jsonString = JSON.stringify(myObj);

        function dataReviver(key, value) {
            if (key == 'lastname') {
                return 'interpreter';
            } else {
                return value;
            }
        }

        var jsonObj = JSON.parse(jsonString, dataReviver);

        JSON.stringify(jsonObj);"#,
    );
    assert_eq!(result, r#"{"firstname":"boa","lastname":"interpreter"}"#);
}
