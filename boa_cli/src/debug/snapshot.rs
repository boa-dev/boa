use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use boa_engine::{
    object::ObjectInitializer,
    snapshot::{Snapshot, SnapshotSerializer},
    Context, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

const SNAPSHOT_PATH: &str = "./snapshot.bin";

fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}

fn create(_: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let Ok(mut file) = OpenOptions::new().write(true).create(true).open(SNAPSHOT_PATH) else {
        return Err(JsNativeError::error().with_message("could not create snapshot.bin file").into());
    };

    let serializer = SnapshotSerializer::new();

    let snapshot = serializer.serialize(context).unwrap();

    file.write_all(snapshot.bytes()).unwrap();
    file.flush().unwrap();

    Ok(JsValue::undefined())
}

fn load(_: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    // let Ok(mut file) = OpenOptions::new().read(true).open(SNAPSHOT_PATH) else {
    //     return Err(JsNativeError::error().with_message("could not open snapshot.bin file").into());
    // };

    let bytes = get_file_as_byte_vec(SNAPSHOT_PATH);

    let snapshot = Snapshot::new(bytes);

    let deser_context = snapshot.deserialize().unwrap();

    *context = deser_context;

    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(create), "create", 0)
        .function(NativeFunction::from_fn_ptr(load), "load", 1)
        .build()
}
