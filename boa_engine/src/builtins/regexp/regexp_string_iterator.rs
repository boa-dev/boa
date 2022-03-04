//! This module implements the global `RegExp String Iterator` object.
//!
//! A `RegExp` String Iterator is an object, that represents a specific iteration over some
//! specific String instance object, matching against some specific `RegExp` instance object.
//! There is not a named constructor for `RegExp` String Iterator objects. Instead, `RegExp`
//! String Iterator objects are created by calling certain methods of `RegExp` instance objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-regexp-string-iterator-objects

use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, regexp},
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use regexp::{advance_string_index, RegExp};

// TODO: See todos in create_regexp_string_iterator and next.
#[derive(Debug, Clone, Finalize, Trace)]
pub struct RegExpStringIterator {
    matcher: JsObject,
    string: JsString,
    global: bool,
    unicode: bool,
    completed: bool,
}

// TODO: See todos in create_regexp_string_iterator and next.
impl RegExpStringIterator {
    fn new(matcher: JsObject, string: JsString, global: bool, unicode: bool) -> Self {
        Self {
            matcher,
            string,
            global,
            unicode,
            completed: false,
        }
    }

    /// `22.2.7.1 CreateRegExpStringIterator ( R, S, global, fullUnicode )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createregexpstringiterator
    pub(crate) fn create_regexp_string_iterator(
        matcher: JsObject,
        string: JsString,
        global: bool,
        unicode: bool,
        context: &mut Context,
    ) -> JsValue {
        // TODO: Implement this with closures and generators.
        //       For now all values of the closure are stored in RegExpStringIterator and the actual closure execution is in `.next()`.

        // 1. Assert: Type(S) is String.
        // 2. Assert: Type(global) is Boolean.
        // 3. Assert: Type(fullUnicode) is Boolean.

        // 4. Let closure be a new Abstract Closure with no parameters that captures R, S, global,
        //    and fullUnicode and performs the following steps when called:

        // 5. Return ! CreateIteratorFromClosure(closure, "%RegExpStringIteratorPrototype%", %RegExpStringIteratorPrototype%).

        let regexp_string_iterator = JsObject::from_proto_and_data(
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .regexp_string_iterator(),
            ObjectData::reg_exp_string_iterator(Self::new(matcher, string, global, unicode)),
        );

        regexp_string_iterator.into()
    }

    pub fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mut iterator = this.as_object().map(JsObject::borrow_mut);
        let iterator = iterator
            .as_mut()
            .and_then(|obj| obj.as_regexp_string_iterator_mut())
            .ok_or_else(|| context.construct_type_error("`this` is not a RegExpStringIterator"))?;
        if iterator.completed {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }

        // TODO: This is the code that should be created as a closure in create_regexp_string_iterator.

        // i. Let match be ? RegExpExec(R, S).
        let m = RegExp::abstract_exec(&iterator.matcher, iterator.string.clone(), context)?;

        if let Some(m) = m {
            // iii. If global is false, then
            if !iterator.global {
                // 1. Perform ? Yield(match).
                // 2. Return undefined.
                iterator.completed = true;
                return Ok(create_iter_result_object(m.into(), false, context));
            }

            // iv. Let matchStr be ? ToString(? Get(match, "0")).
            let m_str = m.get("0", context)?.to_string(context)?;

            // v. If matchStr is the empty String, then
            if m_str.is_empty() {
                // 1. Let thisIndex be ℝ(? ToLength(? Get(R, "lastIndex"))).
                let this_index = iterator
                    .matcher
                    .get("lastIndex", context)?
                    .to_length(context)?;

                // 2. Let nextIndex be ! AdvanceStringIndex(S, thisIndex, fullUnicode).
                let next_index =
                    advance_string_index(&iterator.string, this_index, iterator.unicode);

                // 3. Perform ? Set(R, "lastIndex", 𝔽(nextIndex), true).
                iterator
                    .matcher
                    .set("lastIndex", next_index, true, context)?;
            }

            // vi. Perform ? Yield(match).
            Ok(create_iter_result_object(m.into(), false, context))
        } else {
            // ii. If match is null, return undefined.
            iterator.completed = true;
            Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ))
        }
    }

    /// Create the `%ArrayIteratorPrototype%` object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(
        iterator_prototype: JsObject,
        context: &mut Context,
    ) -> JsObject {
        let _timer = Profiler::global().start_event("RegExp String Iterator", "init");

        // Create prototype
        let result = JsObject::from_proto_and_data(iterator_prototype, ObjectData::ordinary());
        make_builtin_fn(Self::next, "next", &result, 0, context);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = PropertyDescriptor::builder()
            .value("RegExp String Iterator")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        result.insert(to_string_tag, to_string_tag_property);
        result
    }
}
