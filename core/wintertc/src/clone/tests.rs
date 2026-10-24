use crate::test::{TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn failed_serialization_keeps_transfer_buffer_attached() {
    run_test_actions([TestAction::run(indoc! {r#"
        const buffer = new ArrayBuffer(8);
        new Uint8Array(buffer)[0] = 42;

        let threw = false;
        try {
            structuredClone({ buffer, notCloneable() {} }, { transfer: [buffer] });
        } catch {
            threw = true;
        }

        if (!threw || buffer.byteLength !== 8 || new Uint8Array(buffer)[0] !== 42) {
            throw new Error("failed serialization detached or changed the buffer");
        }
    "#})]);
}

#[test]
fn throwing_getter_keeps_transfer_buffer_attached() {
    run_test_actions([TestAction::run(indoc! {r#"
        const buffer = new ArrayBuffer(8);
        const value = {
            buffer,
            get failure() { throw new Error("getter failed"); },
        };

        let threw = false;
        try {
            structuredClone(value, { transfer: [buffer] });
        } catch {
            threw = true;
        }

        if (!threw || buffer.byteLength !== 8) {
            throw new Error("throwing getter detached the buffer");
        }
    "#})]);
}

#[test]
fn successful_transfer_detaches_and_preserves_bytes() {
    run_test_actions([TestAction::run(indoc! {r#"
        const buffer = new ArrayBuffer(8);
        const view = new Uint8Array(buffer);
        view[0] = 42;
        const clone = structuredClone({ buffer, view }, { transfer: [buffer] });

        if (buffer.byteLength !== 0 || clone.buffer.byteLength !== 8 ||
            clone.view.buffer !== clone.buffer || clone.view[0] !== 42) {
            throw new Error("transfer did not preserve the cloned buffer and view");
        }
    "#})]);
}

#[test]
fn unused_transfer_is_detached() {
    run_test_actions([TestAction::run(indoc! {r#"
        const buffer = new ArrayBuffer(8);
        structuredClone({}, { transfer: [buffer] });
        if (buffer.byteLength !== 0) {
            throw new Error("unused transfer buffer was not detached");
        }
    "#})]);
}

#[test]
fn duplicate_transfer_is_rejected_without_detaching() {
    run_test_actions([TestAction::run(indoc! {r#"
        const buffer = new ArrayBuffer(8);
        let threw = false;
        try {
            structuredClone({ buffer }, { transfer: [buffer, buffer] });
        } catch {
            threw = true;
        }

        if (!threw || buffer.byteLength !== 8) {
            throw new Error("duplicate transfer was accepted or detached the buffer");
        }
    "#})]);
}

#[test]
fn buffer_detached_by_getter_does_not_detach_other_transfers() {
    run_test_actions([TestAction::run(indoc! {r#"
        const first = new ArrayBuffer(8);
        const second = new ArrayBuffer(8);
        const value = {
            first,
            get detachSecond() {
                structuredClone(second, { transfer: [second] });
                return true;
            },
        };

        let threw = false;
        try {
            structuredClone(value, { transfer: [first, second] });
        } catch {
            threw = true;
        }

        if (!threw || first.byteLength !== 8 || second.byteLength !== 0) {
            throw new Error("a transfer was detached before all buffers were checked");
        }
    "#})]);
}
