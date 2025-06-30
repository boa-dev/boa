//! Integration tests running the Web Platform Tests (WPT) for the `boa_runtime` crate.
#![allow(unused_crate_dependencies)]
#![cfg(feature = "wpt")]

use boa_engine::class::Class;
use boa_engine::parser::source::UTF16Input;
use boa_engine::property::Attribute;
use boa_engine::value::TryFromJs;
use boa_engine::{
    js_error, js_str, js_string, Context, Finalize, JsData, JsResult, JsString, JsValue, Source,
    Trace,
};
use boa_gc::Gc;
use boa_interop::{ContextData, IntoJsFunctionCopied};
use boa_runtime::url::Url;
use boa_runtime::RegisterOptions;
use logger::RecordingLogEvent;
use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

mod logger;

/// The test status JavaScript type from WPT. This is defined in the test harness.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestStatus {
    Pass = 0,
    Fail = 1,
    Timeout = 2,
    NotRun = 3,
    PreconditionFailed = 4,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Fail => write!(f, "FAIL"),
            Self::Timeout => write!(f, "TIMEOUT"),
            Self::NotRun => write!(f, "NOTRUN"),
            Self::PreconditionFailed => write!(f, "PRECONDITION FAILED"),
        }
    }
}

impl TryFromJs for TestStatus {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_u32(context) {
            Ok(0) => Ok(Self::Pass),
            Ok(1) => Ok(Self::Fail),
            Ok(2) => Ok(Self::Timeout),
            Ok(3) => Ok(Self::NotRun),
            Ok(4) => Ok(Self::PreconditionFailed),
            _ => Err(js_error!("Invalid test status")),
        }
    }
}

/// A single test serialization.
#[derive(TryFromJs)]
struct Test {
    name: JsString,
    status: TestStatus,
    message: Option<JsString>,
    properties: BTreeMap<JsString, JsValue>,
}

/// A Test suite source code.
struct TestSuiteSource {
    path: PathBuf,
    bytes: OnceCell<Vec<u16>>,
}

const REWRITE_RULES: &[(&str, &str)] = &[(
    "/resources/WebIDLParser.js",
    "/resources/webidl2/webidl2.js",
)];

impl TestSuiteSource {
    /// Create a new test suite source.
    fn new(source: impl AsRef<Path>) -> Self {
        Self {
            path: source.as_ref().to_path_buf(),
            bytes: OnceCell::new(),
        }
    }

    fn read_to_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        fn read_string(slice: &[u8], size: usize) -> Option<String> {
            assert!(2 * size <= slice.len());
            let iter = (0..size).map(|i| u16::from_be_bytes([slice[2 * i], slice[2 * i + 1]]));

            std::char::decode_utf16(iter)
                .collect::<Result<String, _>>()
                .ok()
        }
        let buffer = std::fs::read(&self.path)?;
        // Check if buffer contains UTF8 or UTF16.
        let maybe_utf8 = String::from_utf8(buffer.clone());
        if let Ok(utf8) = maybe_utf8 {
            Ok(utf8)
        } else if let Some(utf16) = read_string(&buffer, buffer.len() / 2) {
            Ok(utf16)
        } else {
            Err("Could not determine encoding".into())
        }
    }

    fn source(&self) -> Source<'_, UTF16Input<'_>> {
        let b = self.bytes.get_or_init(|| {
            self.read_to_string()
                .unwrap()
                .encode_utf16()
                .collect::<Vec<u16>>()
        });
        Source::from_utf16(b).with_path(&self.path)
    }

    fn scripts(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut scripts: Vec<String> = Vec::new();
        let dir = self.path.parent().expect("Could not get parent directory");

        'outer: for script in self.meta()?.get("script").unwrap_or(&Vec::new()) {
            let script = script
                .split_once('?')
                .map_or(script.to_string(), |(s, _)| s.to_string());

            // Resolve the source path relative to the script path, but under the wpt_path.
            let script_path = Path::new(&script);
            let path = if script_path.is_relative() {
                dir.join(script_path)
            } else {
                script_path.to_path_buf()
            };

            for (from, to) in REWRITE_RULES {
                if path.to_string_lossy().as_ref() == *from {
                    scripts.push((*to).to_string());
                    continue 'outer;
                }
            }
            scripts.push(path.to_string_lossy().to_string());
        }
        Ok(scripts)
    }

    fn meta(&self) -> Result<BTreeMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut meta: BTreeMap<String, Vec<String>> = BTreeMap::new();

        // Read the whole file and extract the metadata.
        let content = self.read_to_string()?;
        for line in content.lines() {
            if let Some(kv) = line.strip_prefix("// META:") {
                let kv = kv.trim();
                if let Some((key, value)) = kv.split_once('=') {
                    meta.entry(key.to_string())
                        .or_default()
                        .push(value.to_string());
                }
            } else if !line.starts_with("//") && !line.is_empty() {
                break;
            }
        }

        Ok(meta)
    }
}

/// Create the BOA context and add the necessary global objects for WPT.
fn create_context(wpt_path: &Path) -> (Context, logger::RecordingLogger) {
    let mut context = Context::default();
    let logger = logger::RecordingLogger::new();
    boa_runtime::register(
        &mut context,
        RegisterOptions::new().with_console_logger(logger.clone()),
    )
    .expect("Failed to register boa_runtime");

    // Define self as the globalThis.
    let global_this = context.global_object();
    context
        .register_global_property(js_str!("self"), global_this, Attribute::all())
        .unwrap();

    // Define location to be an empty URL.
    let location = Url::new("about:blank".to_string().into(), None.into())
        .expect("Could not parse the location URL");
    let location =
        Url::from_data(location, &mut context).expect("Could not create the location URL");
    context
        .register_global_property(js_str!("location"), location, Attribute::all())
        .unwrap();

    let harness_path = wpt_path.join("resources/testharness.js");
    let harness = Source::from_filepath(&harness_path).expect("Could not create source.");

    if let Err(e) = context.eval(harness) {
        panic!("Failed to eval testharness.js: {e:#?}");
    }

    (context, logger)
}

/// The result callback for the WPT test.
fn result_callback__(
    ContextData(logger): ContextData<logger::RecordingLogger>,
    test: Test,
    context: &mut Context,
) -> JsResult<()> {
    // Check the logs if the test succeeded.
    assert_eq!(
        test.status,
        TestStatus::Pass,
        "Test {:?} failed with message:\n  {:?}",
        test.name.to_std_string_lossy(),
        test.message.unwrap_or_default()
    );

    // Check the logs.
    let logs = logger.all_logs();
    if let Some(log_regex) = test.properties.get(&js_string!("logs")) {
        if let Ok(logs_re) = log_regex.try_js_into::<Vec<JsValue>>(context) {
            for re in logs_re {
                let passes = if let Some(re) = re.as_regexp() {
                    logs.iter().any(|log: &RecordingLogEvent| -> bool {
                        let s = JsString::from(log.msg.clone());
                        re.test(s, context).unwrap_or(false)
                    })
                } else {
                    let re_str = re.to_string(context)?.to_std_string_escaped();
                    logs.iter()
                        .any(|log: &RecordingLogEvent| -> bool { log.msg.contains(&re_str) })
                };
                assert!(
                    passes,
                    "Test {:?} failed to find log: {}",
                    test.name.to_std_string_lossy(),
                    re.display()
                );
            }
        }
    }

    Ok(())
}

fn complete_callback__(ContextData(test_done): ContextData<TestCompletion>) {
    test_done.done();
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct TestCompletion(Gc<AtomicBool>);

impl TestCompletion {
    fn new() -> Self {
        Self(Gc::new(AtomicBool::new(false)))
    }

    fn done(&self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    fn is_done(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Load and execute the test file.
// This can be marked as allow unused because it would give false positives
// in clippy.
#[allow(unused)]
fn execute_test_file(path: &Path) {
    let dir = path.parent().unwrap();
    let wpt_path = PathBuf::from(
        std::env::var("WPT_ROOT").expect("Could not find WPT_ROOT environment variable"),
    );
    let (mut context, logger) = create_context(&wpt_path);
    let test_done = TestCompletion::new();

    // Insert the logger to be able to access the logs after the test is done.
    context.insert_data(logger.clone());
    context.insert_data(test_done.clone());

    let function = result_callback__
        .into_js_function_copied(&mut context)
        .to_js_function(context.realm());
    context
        .register_global_property(js_str!("result_callback__"), function, Attribute::all())
        .expect("Could not register result_callback__");
    context
        .eval(Source::from_bytes(
            b"add_result_callback(result_callback__);",
        ))
        .expect("Could not eval add_result_callback");

    let function = complete_callback__
        .into_js_function_copied(&mut context)
        .to_js_function(context.realm());
    context
        .register_global_property(js_str!("complete_callback__"), function, Attribute::all())
        .expect("Could not register complete_callback__");
    context
        .eval(Source::from_bytes(
            b"add_completion_callback(complete_callback__);",
        ))
        .expect("Could not eval add_completion_callback");

    // Load the test.
    let source = TestSuiteSource::new(path);
    for script in source.scripts().expect("Could not get scripts") {
        // Resolve the source path relative to the script path, but under the wpt_path.
        let script_path = Path::new(&script);
        let path = if script_path.is_relative() {
            dir.join(script_path)
        } else {
            wpt_path.join(script_path.to_string_lossy().trim_start_matches('/'))
        };

        if path.exists() {
            let source = Source::from_filepath(&path).expect("Could not parse source.");
            if let Err(err) = context.eval(source) {
                panic!("Could not eval script, path = {path:?}, err = {err:?}");
            }
        } else {
            panic!("Script does not exist, path = {path:?}");
        }
    }
    context
        .eval(source.source())
        .expect("Could not evaluate source");
    context.run_jobs();

    // Done()
    context
        .eval(Source::from_bytes(b"done()"))
        .expect("Done unexpectedly threw an error.");

    let start = std::time::Instant::now();
    while !test_done.is_done() {
        context.run_jobs();

        assert!(
            start.elapsed().as_secs() < 10,
            "Test did not complete in 10 seconds."
        );
    }
}

/// Test the console with the WPT test suite.
#[cfg(not(clippy))]
#[rstest::rstest]
fn console(
    #[base_dir = "${WPT_ROOT}"]
    #[files("console/*.any.js")]
    // TODO: The console-log-large-array.any.js test is too slow.
    #[exclude("console-log-large-array.any.js")]
    #[exclude("idlharness")]
    path: PathBuf,
) {
    execute_test_file(&path);
}

/// Test the text encoder/decoder with the WPT test suite.
#[cfg(not(clippy))]
#[ignore] // TODO: support all encodings.
#[rstest::rstest]
fn encoding(
    #[base_dir = "${WPT_ROOT}"]
    #[files("encoding/textdecoder-*.any.js")]
    #[exclude("idlharness")]
    path: PathBuf,
) {
    execute_test_file(&path);
}

/// Test the URL class with the WPT test suite.
// A bunch of these tests are failing due to lack of support in the URL class,
// or missing APIs such as fetch.
#[cfg(not(clippy))]
#[rstest::rstest]
fn url(
    #[base_dir = "${WPT_ROOT}"]
    #[files("url/url-*.any.js")]
    #[exclude("idlharness")]
    // "Base URL about:blank cannot be a base"
    #[exclude("url-searchparams.any.js")]
    // "fetch is not defined"
    #[exclude("url-origin.any.js")]
    #[exclude("url-setters.any.js")]
    #[exclude("url-constructor.any.js")]
    path: PathBuf,
) {
    execute_test_file(&path);
}
