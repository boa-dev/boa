#[cfg(feature = "gc-stats")]
use boa_engine::property::Attribute;
use boa_engine::{
    Context, JsObject, JsResult, JsValue, NativeFunction, js_string, object::ObjectInitializer,
};

/// Trigger garbage collection.
fn collect(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    boa_gc::force_collect();
    Ok(JsValue::undefined())
}

/// Get the garbage collector statistics of the current thread.
///
/// Timings are reported in milliseconds.
#[cfg(feature = "gc-stats")]
fn stats(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let stats = boa_gc::stats();
    let ms = |duration: std::time::Duration| duration.as_secs_f64() * 1000.0;

    let object = ObjectInitializer::new(context)
        .property(
            js_string!("collections"),
            stats.collections,
            Attribute::all(),
        )
        .property(
            js_string!("finalizingCollections"),
            stats.finalizing_collections,
            Attribute::all(),
        )
        .property(
            js_string!("totalBytesAllocated"),
            stats.total_bytes_allocated,
            Attribute::all(),
        )
        .property(
            js_string!("totalBytesFreed"),
            stats.total_bytes_freed,
            Attribute::all(),
        )
        .property(
            js_string!("objectsAllocated"),
            stats.objects_allocated,
            Attribute::all(),
        )
        .property(
            js_string!("objectsSwept"),
            stats.objects_swept,
            Attribute::all(),
        )
        .property(
            js_string!("bytesAllocated"),
            stats.bytes_allocated,
            Attribute::all(),
        )
        .property(
            js_string!("traceNonRootsMs"),
            ms(stats.trace_non_roots_time),
            Attribute::all(),
        )
        .property(js_string!("markMs"), ms(stats.mark_time), Attribute::all())
        .property(
            js_string!("finalizeMs"),
            ms(stats.finalize_time),
            Attribute::all(),
        )
        .property(
            js_string!("remarkMs"),
            ms(stats.remark_time),
            Attribute::all(),
        )
        .property(
            js_string!("sweepMs"),
            ms(stats.sweep_time),
            Attribute::all(),
        )
        .property(
            js_string!("weakMapPruneMs"),
            ms(stats.weak_map_prune_time),
            Attribute::all(),
        )
        .property(
            js_string!("totalMs"),
            ms(stats.total_time),
            Attribute::all(),
        )
        .build();

    Ok(object.into())
}

/// Reset the garbage collector statistics of the current thread.
#[cfg(feature = "gc-stats")]
fn reset_stats(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    boa_gc::reset_stats();
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    let mut object = ObjectInitializer::new(context);
    object.function(
        NativeFunction::from_fn_ptr(collect),
        js_string!("collect"),
        0,
    );

    #[cfg(feature = "gc-stats")]
    object
        .function(NativeFunction::from_fn_ptr(stats), js_string!("stats"), 0)
        .function(
            NativeFunction::from_fn_ptr(reset_stats),
            js_string!("resetStats"),
            0,
        );

    object.build()
}
