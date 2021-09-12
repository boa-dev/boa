//! This module implements the global `RegExp String Iterator` object.
//!
//! A RegExp String Iterator is an object, that represents a specific iteration over some specific String instance object, matching against some specific RegExp instance object.
//! There is not a named constructor for RegExp String Iterator objects.
//! Instead, RegExp String Iterator objects are created by calling certain methods of RegExp instance objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-regexp-string-iterator-objects

use regexp::{advance_string_index, RegExp};

use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object, regexp},
    gc::{Finalize, Trace},
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult, JsString, JsValue,
};

// TODO: See todos in create_regexp_string_iterator and next.
#[derive(Debug, Clone, Finalize, Trace)]
pub struct RegExpStringIterator {
    matcher: JsValue,
    string: JsString,
    global: bool,
    unicode: bool,
    completed: bool,
}

// TODO: See todos in create_regexp_string_iterator and next.
impl RegExpStringIterator {
    fn new(matcher: JsValue, string: JsString, global: bool, unicode: bool) -> Self {
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
        matcher: &JsValue,
        string: JsString,
        global: bool,
        unicode: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // TODO: Implement this with closures and generators.
        //       For now all values of the closure are stored in RegExpStringIterator and the actual closure execution is in `.next()`.

        // 1. Assert: Type(S) is String.
        // 2. Assert: Type(global) is Boolean.
        // 3. Assert: Type(fullUnicode) is Boolean.

        // 4. Let closure be a new Abstract Closure with no parameters that captures R, S, global,
        //    and fullUnicode and performs the following steps when called:

        // 5. Return ! CreateIteratorFromClosure(closure, "%RegExpStringIteratorPrototype%", %RegExpStringIteratorPrototype%).
        let regexp_string_iterator = JsValue::new_object(context);
        regexp_string_iterator.set_data(ObjectData::reg_exp_string_iterator(Self::new(
            matcher.clone(),
            string,
            global,
            unicode,
        )));
        regexp_string_iterator
            .as_object()
            .expect("regexp string iterator object")
            .set_prototype_instance(
                context
                    .iterator_prototypes()
                    .regexp_string_iterator()
                    .into(),
            );

        Ok(regexp_string_iterator)
    }

    pub fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let JsValue::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(iterator) = object.as_regexp_string_iterator_mut() {
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
                            .get_field("lastIndex", context)?
                            .to_length(context)?;

                        // 2. Let nextIndex be ! AdvanceStringIndex(S, thisIndex, fullUnicode).
                        let next_index = advance_string_index(
                            iterator.string.clone(),
                            this_index,
                            iterator.unicode,
                        );

                        // 3. Perform ? Set(R, "lastIndex", 𝔽(nextIndex), true).
                        iterator
                            .matcher
                            .set_field("lastIndex", next_index, true, context)?;
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
            } else {
                context.throw_type_error("`this` is not a RegExpStringIterator")
            }
        } else {
            context.throw_type_error("`this` is not a RegExpStringIterator")
        }
    }

    /// Create the %ArrayIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(iterator_prototype: JsValue, context: &mut Context) -> JsObject {
        let _timer = BoaProfiler::global().start_event("RegExp String Iterator", "init");

        // Create prototype
        let result = context.construct_object();
        make_builtin_fn(Self::next, "next", &result, 0, context);
        result.set_prototype_instance(iterator_prototype);

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
