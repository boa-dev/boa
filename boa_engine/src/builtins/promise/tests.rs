use crate::{forward, Context};

#[test]
fn promise() {
    let mut context = Context::default();
    let init = r#"
        let count = 0;
        const promise = new Promise((resolve, reject) => {
        count += 1;
        resolve(undefined);
        }).then((_) => (count += 1));
        count += 1;
        count;
        "#;
    let result = context.eval(init).unwrap();
    assert_eq!(result.as_number(), Some(2_f64));
    let after_completion = forward(&mut context, "count");
    assert_eq!(after_completion, String::from("3"));
}
