use std::{fs::OpenOptions, io::Write};

use boa_engine::{
    object::ObjectInitializer, snapshot::SnapshotSerializer, Context, JsNativeError, JsObject,
    JsResult, JsValue, NativeFunction,
};

const SNAPSHOT_PATH: &str = "./snapshot.bin";

fn create(_: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let Ok(mut file) = OpenOptions::new().write(true).create(true).open(SNAPSHOT_PATH) else {
        return Err(JsNativeError::error().with_message("could not create snapshot.bin file").into());
    };

    let mut serializer = SnapshotSerializer::new();

    serializer.serialize(context).unwrap();

    file.write_all(serializer.bytes()).unwrap();
    file.flush().unwrap();

    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(create), "create", 0)
        .build()
}
