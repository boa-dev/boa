#[test]
fn compile_bytecode() {
    use boa_engine::vm::{VirtualMachineTracer, trace::VirtualMachineEvent};
    use boa_engine::{Context, Source};
    use insta::glob;
    use std::fmt::Write;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug, Clone)]
    pub struct SnapshotTracer {
        inner: Rc<RefCell<String>>,
    }

    impl SnapshotTracer {
        fn new(inner: Rc<RefCell<String>>) -> Self {
            Self { inner }
        }
    }

    impl VirtualMachineTracer for SnapshotTracer {
        fn emit_event(&self, event: VirtualMachineEvent) {
            if let VirtualMachineEvent::CallFrameTrace(call_frame_message) = event {
                let mut out = self.inner.borrow_mut();
                writeln!(&mut *out, "{}", call_frame_message.bytecode).unwrap();
            }
        }
    }

    glob!("../scripts/", "**/*.js", |path| {
        let trace_sink = Rc::new(RefCell::new(String::new()));
        let context = &mut Context::default();
        context.set_trace(true);
        context.set_virtual_machine_tracer(Box::new(SnapshotTracer::new(trace_sink.clone())));
        let source = Source::from_filepath(path).expect("Could not load source");
        let result = match context.eval(source) {
            Ok(v) => v.display().to_string(),
            Err(e) => format!("{e}"),
        };
        {
            let mut sink = trace_sink.borrow_mut();
            writeln!(&mut sink, "Evaluation result: {result}").unwrap();
        }
        insta::assert_snapshot!(*trace_sink.borrow());
    });
}
