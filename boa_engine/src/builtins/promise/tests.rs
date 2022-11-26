use boa_parser::Source;

use crate::{context::ContextBuilder, forward, job::SimpleJobQueue};

#[test]
fn promise() {
    let queue = SimpleJobQueue::new();
    let mut context = ContextBuilder::new().job_queue(&queue).build().unwrap();
    let init = r#"
        let count = 0;
        const promise = new Promise((resolve, reject) => {
            count += 1;
            resolve(undefined);
        }).then((_) => (count += 1));
        count += 1;
        count;
    "#;
    let result = context.eval_script(Source::from_bytes(init)).unwrap();
    assert_eq!(result.as_number(), Some(2_f64));
    context.run_jobs();
    let after_completion = forward(&mut context, "count");
    assert_eq!(after_completion, String::from("3"));
}
