//! This module implements the `ForInIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-for-in-iterator-objects

// TODO: This should not be a builtin, since this cannot be seen by ECMAScript code per the spec.
// Opportunity to optimize this for iteration speed.

use crate::{
    builtins::{iterable::create_iter_result_object, BuiltInBuilder, IntrinsicObject},
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    object::{JsObject, ObjectData},
    property::PropertyKey,
    realm::Realm,
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// The `ForInIterator` object represents an iteration over some specific object.
/// It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-for-in-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct ForInIterator {
    object: JsValue,
    visited_keys: FxHashSet<JsString>,
    remaining_keys: VecDeque<JsString>,
    object_was_visited: bool,
}

impl IntrinsicObject for ForInIterator {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event("ForInIterator", "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(
                realm
                    .intrinsics()
                    .objects()
                    .iterator_prototypes()
                    .iterator(),
            )
            .static_method(Self::next, "next", 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().for_in()
    }
}

impl ForInIterator {
    fn new(object: JsValue) -> Self {
        Self {
            object,
            visited_keys: FxHashSet::default(),
            remaining_keys: VecDeque::default(),
            object_was_visited: false,
        }
    }

    /// `CreateForInIterator( object )`
    ///
    /// Creates a new iterator over the given object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createforiniterator
    pub(crate) fn create_for_in_iterator(object: JsValue, context: &Context<'_>) -> JsObject {
        JsObject::from_proto_and_data(
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .for_in(),
            ObjectData::for_in_iterator(Self::new(object)),
        )
    }

    /// %ForInIteratorPrototype%.next( )
    ///
    /// Gets the next result in the object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%foriniteratorprototype%.next
    pub(crate) fn next(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let mut iterator = this.as_object().map(JsObject::borrow_mut);
        let iterator = iterator
            .as_mut()
            .and_then(|obj| obj.as_for_in_iterator_mut())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a ForInIterator"))?;
        let mut object = iterator.object.to_object(context)?;
        loop {
            if !iterator.object_was_visited {
                let keys = object.__own_property_keys__(context)?;
                for k in keys {
                    match k {
                        PropertyKey::String(ref k) => {
                            iterator.remaining_keys.push_back(k.clone());
                        }
                        PropertyKey::Index(i) => {
                            iterator.remaining_keys.push_back(i.to_string().into());
                        }
                        PropertyKey::Symbol(_) => {}
                    }
                }
                iterator.object_was_visited = true;
            }
            while let Some(r) = iterator.remaining_keys.pop_front() {
                if !iterator.visited_keys.contains(&r) {
                    if let Some(desc) =
                        object.__get_own_property__(&PropertyKey::from(r.clone()), context)?
                    {
                        iterator.visited_keys.insert(r.clone());
                        if desc.expect_enumerable() {
                            return Ok(create_iter_result_object(JsValue::new(r), false, context));
                        }
                    }
                }
            }
            let proto = object.prototype().clone();
            match proto {
                Some(o) => {
                    object = o;
                }
                _ => {
                    return Ok(create_iter_result_object(
                        JsValue::undefined(),
                        true,
                        context,
                    ))
                }
            }
            iterator.object = JsValue::new(object.clone());
            iterator.object_was_visited = false;
        }
    }
}
