use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{self, Sender},
    thread::JoinHandle,
    time::Duration,
};

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, Source,
    builtins::array_buffer::{ArrayBuffer, SharedArrayBuffer},
    js_string,
    native_function::NativeFunction,
    object::{JsObject, ObjectInitializer, builtins::JsSharedArrayBuffer},
    property::Attribute,
};
use bus::BusReader;

use crate::START;

pub(super) enum WorkerResult {
    Ok,
    Err(String),
    Panic(String),
}

pub(super) type WorkerHandle = JoinHandle<Result<(), String>>;

#[derive(Debug, Clone)]
pub(super) struct WorkerHandles(Rc<RefCell<Vec<WorkerHandle>>>);

impl WorkerHandles {
    pub(super) fn new() -> Self {
        Self(Rc::default())
    }

    pub(super) fn join_all(&mut self) -> Vec<WorkerResult> {
        let handles = std::mem::take(&mut *self.0.borrow_mut());

        handles
            .into_iter()
            .map(|h| {
                let result = h.join();

                match result {
                    Ok(Ok(())) => WorkerResult::Ok,
                    Ok(Err(msg)) => {
                        eprintln!("Detected error on worker thread: {msg}");
                        WorkerResult::Err(msg)
                    }
                    Err(e) => {
                        let msg = e
                            .downcast_ref::<&str>()
                            .map(|&s| String::from(s))
                            .unwrap_or_default();
                        eprintln!("Detected panic on worker thread: {msg}");
                        WorkerResult::Panic(msg)
                    }
                }
            })
            .collect()
    }
}

impl Drop for WorkerHandles {
    fn drop(&mut self) {
        self.join_all();
    }
}

/// Creates the object $262 in the context.
pub(super) fn register_js262(handles: WorkerHandles, context: &mut Context) -> JsObject {
    let global_obj = context.global_object();

    let agent = agent_obj(handles, context);

    let js262 = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(create_realm),
            js_string!("createRealm"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(detach_array_buffer),
            js_string!("detachArrayBuffer"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(eval_script),
            js_string!("evalScript"),
            1,
        )
        .function(NativeFunction::from_fn_ptr(gc), js_string!("gc"), 0)
        .property(
            js_string!("global"),
            global_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("agent"),
            agent,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .build();

    context
        .register_global_property(
            js_string!("$262"),
            js262.clone(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .expect("shouldn't fail with the default global");

    js262
}

/// The `$262.createRealm()` function.
///
/// Creates a new ECMAScript Realm, defines this API on the new realm's global object, and
/// returns the `$262` property of the new realm's global object.
#[allow(clippy::unnecessary_wraps)]
fn create_realm(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let context = &mut Context::default();

    let js262 = register_js262(WorkerHandles::new(), context);

    Ok(JsValue::new(js262))
}

/// The `$262.detachArrayBuffer()` function.
///
/// Implements the `DetachArrayBuffer` abstract operation.
fn detach_array_buffer(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    fn type_err() -> JsNativeError {
        JsNativeError::typ().with_message("The provided object was not an ArrayBuffer")
    }

    // 1. Assert: IsSharedArrayBuffer(arrayBuffer) is false.
    let object = args.first().and_then(JsValue::as_object);
    let mut array_buffer = object
        .as_ref()
        .and_then(|o| o.downcast_mut::<ArrayBuffer>())
        .ok_or_else(type_err)?;

    // 2. If key is not present, set key to undefined.
    let key = args.get_or_undefined(1);

    // 3. If SameValue(arrayBuffer.[[ArrayBufferDetachKey]], key) is false, throw a TypeError exception.
    // 4. Set arrayBuffer.[[ArrayBufferData]] to null.
    // 5. Set arrayBuffer.[[ArrayBufferByteLength]] to 0.
    array_buffer.detach(key)?;

    // 6. Return NormalCompletion(null).
    Ok(JsValue::null())
}

/// The `$262.evalScript()` function.
///
/// Accepts a string value as its first argument and executes it as an ECMAScript script.
fn eval_script(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    args.first().and_then(JsValue::as_string).map_or_else(
        || Ok(JsValue::undefined()),
        |source_text| context.eval(Source::from_bytes(&source_text.to_std_string_escaped())),
    )
}

/// The `$262.gc()` function.
///
/// Wraps the host's garbage collection invocation mechanism, if such a capability exists.
/// Must throw an exception if no capability exists. This is necessary for testing the
/// semantics of any feature that relies on garbage collection, e.g. the `WeakRef` API.
#[allow(clippy::unnecessary_wraps)]
fn gc(_this: &JsValue, _: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    boa_gc::force_collect();
    Ok(JsValue::undefined())
}

/// The `$262.agent.sleep()` function.
fn sleep(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let ms = args.get_or_undefined(0).to_number(context)? / 1000.0;
    std::thread::sleep(Duration::from_secs_f64(ms));
    Ok(JsValue::undefined())
}

/// The `$262.agent.monotonicNow()` function.
#[allow(clippy::unnecessary_wraps)]
fn monotonic_now(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let clock = START
        .get()
        .ok_or_else(|| JsNativeError::typ().with_message("could not get the monotonic clock"))?;
    Ok(JsValue::from(clock.elapsed().as_millis() as f64))
}

/// Initializes the `$262.agent` object in the main agent.
fn agent_obj(handles: WorkerHandles, context: &mut Context) -> JsObject {
    // TODO: improve initialization of this by using a `[[HostDefined]]` field on `Context`.
    let bus = Rc::new(RefCell::new(bus::Bus::new(1)));

    let (reports_tx, reports_rx) = mpsc::channel();

    let start = unsafe {
        let bus = bus.clone();
        NativeFunction::from_closure(move |_, args, context| {
            let script = args
                .get_or_undefined(0)
                .to_string(context)?
                .to_std_string()
                .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;

            let rx = bus.borrow_mut().add_rx();
            let tx = reports_tx.clone();

            handles.0.borrow_mut().push(std::thread::spawn(move || {
                let context = &mut Context::builder()
                    .can_block(true)
                    .build()
                    .map_err(|e| e.to_string())?;
                register_js262_worker(rx, tx, context);

                let src = Source::from_bytes(&script);
                context.eval(src).map_err(|e| e.to_string())?;

                Ok(())
            }));

            Ok(JsValue::undefined())
        })
    };

    let broadcast = unsafe {
        // should technically also have a second numeric argument, but the test262 never uses it.
        NativeFunction::from_closure(move |_, args, _| {
            let buffer = args.get_or_undefined(0).as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("argument was not a shared array")
            })?;
            let buffer = buffer
                .downcast_ref::<SharedArrayBuffer>()
                .ok_or_else(|| {
                    JsNativeError::typ().with_message("argument was not a shared array")
                })?
                .clone();

            bus.borrow_mut().broadcast(buffer);

            Ok(JsValue::undefined())
        })
    };

    let get_report = unsafe {
        NativeFunction::from_closure(move |_, _, _| {
            let Ok(msg) = reports_rx.try_recv() else {
                return Ok(JsValue::null());
            };

            Ok(js_string!(&msg[..]).into())
        })
    };

    ObjectInitializer::new(context)
        .function(start, js_string!("start"), 1)
        .function(broadcast, js_string!("broadcast"), 2)
        .function(get_report, js_string!("getReport"), 0)
        .function(NativeFunction::from_fn_ptr(sleep), js_string!("sleep"), 1)
        .function(
            NativeFunction::from_fn_ptr(monotonic_now),
            js_string!("monotonicNow"),
            0,
        )
        .build()
}

/// Initializes the `$262` object in a worker agent.
fn register_js262_worker(
    rx: BusReader<SharedArrayBuffer>,
    tx: Sender<Vec<u16>>,
    context: &mut Context,
) {
    let rx = RefCell::new(rx);
    let receive_broadcast = unsafe {
        // should technically also have a second numeric argument, but the test262 never uses it.
        NativeFunction::from_closure(move |_, args, context| {
            let array = rx.borrow_mut().recv().map_err(|err| {
                JsNativeError::typ().with_message(format!("failed to receive buffer: {err}"))
            })?;

            let callable = args
                .get_or_undefined(0)
                .as_callable()
                .ok_or_else(|| JsNativeError::typ().with_message("argument is not callable"))?;

            let buffer = JsSharedArrayBuffer::from_buffer(array, context);
            callable.call(&JsValue::undefined(), &[buffer.into()], context)
        })
    };

    let report = unsafe {
        NativeFunction::from_closure(move |_, args, context| {
            let string = args.get_or_undefined(0).to_string(context)?.to_vec();
            tx.send(string)
                .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
            Ok(JsValue::undefined())
        })
    };

    let agent = ObjectInitializer::new(context)
        .function(receive_broadcast, js_string!("receiveBroadcast"), 1)
        .function(report, js_string!("report"), 1)
        .function(NativeFunction::from_fn_ptr(sleep), js_string!("sleep"), 1)
        // Don't need to signal leaving, the main thread will join with the worker
        // threads anyways.
        .function(
            NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
            js_string!("leaving"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(monotonic_now),
            js_string!("monotonicNow"),
            0,
        )
        .build();

    let js262 = ObjectInitializer::new(context)
        .property(
            js_string!("agent"),
            agent,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .build();

    context
        .register_global_property(
            js_string!("$262"),
            js262,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .expect("shouldn't fail with the default global");
}
