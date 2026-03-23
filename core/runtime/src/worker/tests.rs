use super::Worker;
use crate::extensions::WorkerExtension;
use crate::test::TestAction;

#[test]
fn basic_worker_spawn() {
    let script = r#"
        let a = 1 + 2;
        if (a !== 3) {
            throw new Error("Math is broken");
        }
    "#;

    // Create a temporary file for the worker script
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("worker_test.js");
    std::fs::write(&script_path, script).expect("Failed to write test script");

    let actions = vec![
        TestAction::run(format!(
            r#"
            let worker = new Worker("{}");
            "#,
            script_path.to_string_lossy().replace("\\", "\\\\") // Escape path for JS string
        )),
        TestAction::run(r#"
            worker.postMessage("test message");
            worker.terminate();
        "#),
    ];

    crate::test::run_test_actions_with(
        actions,
        &mut boa_engine::Context::default(),
    );
}
