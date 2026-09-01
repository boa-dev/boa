//! Boa's implementation of the ECMAScript `Iterator` constructor.
//!
//! The `Iterator` constructor is designed to be subclassed. It may be used as the
//! value of an extends clause of a class definition.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-iterator-constructor

use std::collections::VecDeque;

use crate::{
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
        iterable::iterator_helper::{self, IterableRecord},
        object::OrdinaryObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_error, js_string,
    object::{JsFunction, JsObject, PROTOTYPE, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

use super::{
    get_iterator_flattenable, iterator_helper::IteratorHelper,
    wrap_for_valid_iterator::WrapForValidIterator,
};

use super::{
    IteratorHint,
    iterator_helper::{ZipMode, ZipResultKind},
};

use crate::{JsVariant, builtins::options::get_options_object, property::PropertyKey};

/// [`IfAbruptCloseIterators ( value, iteratorRecords )`][spec]
///
/// `IfAbruptCloseIterators` is a shorthand for a sequence of algorithm steps that
/// use a list of Iterator Records.
///
///  [spec]: https://tc39.es/proposal-joint-iteration/#sec-ifabruptcloseiterators
macro_rules! if_abrupt_close_iterators {
    ($value:expr, $iterators:expr, $context:expr) => {
        // 1. Assert: value is a Completion Record.
        match $value {
            // 2. If value is an abrupt completion, return ? IteratorCloseAll(iteratorRecords, value).
            Err(err) => {
                let mut completion = Err(err);
                // 1. For each element iterator of iterators, in reverse List order, do
                for iterator in $iterators.rev() {
                    // 1.a. Set completion to Completion(IteratorClose(iterator, completion)).
                    completion = iterator.close(completion, $context);
                }
                return match completion {
                    Ok(_) => Err($crate::PanicError::new(
                        "closing an iterator with an error should yield the error",
                    )
                    .into()),
                    Err(err) => Err(err),
                };
            },
            // 3. Else, set value to value.[[Value]].
            Ok(value) => value,
        }
    };
}

/// The `Iterator` constructor.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-constructor
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub(crate) struct IteratorConstructor;

impl IntrinsicObject for IteratorConstructor {
    fn init(realm: &Realm) {
        let iterator_prototype = realm.intrinsics().constructors().iterator().prototype();
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(iterator_prototype.clone()))
            // Static methods
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::concat, js_string!("concat"), 0)
            .static_method(Self::zip, js_string!("zip"), 1)
            .static_method(Self::zip_keyed, js_string!("zipKeyed"), 1)
            .static_property(PROTOTYPE, iterator_prototype, Attribute::empty())
            .build_without_prototype();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().iterator().constructor()
    }
}

impl BuiltInObject for IteratorConstructor {
    const NAME: JsString = StaticJsStrings::ITERATOR;
}

impl BuiltInConstructor for IteratorConstructor {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 5;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::iterator;

    /// `Iterator ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator
    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined or the active function object, throw a TypeError exception.
        if new_target.is_undefined()
            || new_target
                == &context
                    .active_function_object()
                    .unwrap_or_else(|| context.intrinsics().constructors().iterator().constructor())
                    .into()
        {
            return Err(JsNativeError::typ()
                .with_message(if new_target.is_undefined() {
                    "Iterator constructor requires 'new'"
                } else {
                    "Abstract class Iterator not directly constructable"
                })
                .into());
        }

        // 2. Return ? OrdinaryCreateFromConstructor(NewTarget, "%Iterator.prototype%").
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::iterator, context)?;

        // Create an ordinary object (Iterator instances have no internal data slots).
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrdinaryObject,
        )
        .upcast()
        .into())
    }
}

impl IteratorConstructor {
    /// `Iterator.from ( O )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.from
    fn from(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = args.get_or_undefined(0);

        // 1. Let iteratorRecord be ? GetIteratorFlattenable(O, iterate-strings).
        let iterator_record = get_iterator_flattenable(o, true, context)?;

        // 2. Let hasInstance be ? OrdinaryHasInstance(%Iterator%, iteratorRecord.[[Iterator]]).
        let iterator_constructor = context.intrinsics().constructors().iterator().constructor();
        let has_instance = JsValue::ordinary_has_instance(
            &iterator_constructor.clone().into(),
            &iterator_record.iterator().clone().into(),
            context,
        )?;

        // 3. If hasInstance is true, then
        if has_instance {
            // a. Return iteratorRecord.[[Iterator]].
            return Ok(iterator_record.iterator().clone().into());
        }

        // 4. Let wrapper be OrdinaryObjectCreate(%WrapForValidIteratorPrototype%, « [[Iterated]] »).
        // 5. Set wrapper.[[Iterated]] to iteratorRecord.
        // 6. Return wrapper.
        let wrapper = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .wrap_for_valid_iterator(),
            WrapForValidIterator {
                iterated: iterator_record,
            },
        );

        Ok(wrapper.into())
    }

    /// `Iterator.concat ( ...items )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.concat
    fn concat(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let iterables be a new empty List.
        let mut iterables = VecDeque::with_capacity(args.len());

        // 2. For each element item of items, do
        for item in args {
            // a. If item is not an Object, throw a TypeError exception.
            let Some(item) = item.as_object() else {
                return Err(js_error!(TypeError: "Iterator.concat requires iterable objects"));
            };

            // b. Let method be ? GetMethod(item, %Symbol.iterator%).
            // c. If method is undefined, throw a TypeError exception.
            let method = item.get_method(JsSymbol::iterator(), context)?.ok_or_else(
                || js_error!(TypeError: "Iterator.concat requires objects with @@iterator"),
            )?;

            // d. Append the Record { [[OpenMethod]]: method, [[Iterable]]: item } to iterables.
            iterables.push_back(IterableRecord {
                iterable: item,
                open_method: JsFunction::from_object_unchecked(method),
            });
        }

        // 3. Let closure be a new Abstract Closure with no parameters that captures iterables
        //    and performs the following steps when called:
        //    (implemented via IteratorHelperOp::Concat in execute_next)
        // 4-5. Let result be CreateIteratorFromClosure(closure, "Iterator Helper", ...)
        //      with [[UnderlyingIterators]] set to a new empty List.
        let helper = IteratorHelper::create(iterator_helper::Concat::new(iterables), context);

        // 6. Return result.
        Ok(helper.into())
    }

    // ==================== Static Methods — Experimental ====================

    /// `Iterator.zip ( iterables [ , options ] )`
    ///
    /// More information:
    ///  - [TC39 proposal][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.zip
    fn zip(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterables = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. If iterables is not an Object, throw a TypeError exception.
        let iterables = iterables
            .as_object()
            .ok_or_else(|| js_error!(TypeError: "Iterator.zip requires an iterable object"))?;

        // 2 - 7 in options.
        let (mode, padding_option) = parse_options(options, context)?;

        // 8. Let iters be a new empty List.
        let mut iters: Vec<super::IteratorRecord> = Vec::new();

        // 9. Let padding be a new empty List.
        // (padding list built later in build_padding)

        // 10. Let inputIter be ? GetIterator(iterables, sync).
        let mut input_iter = iterables.get_iterator(IteratorHint::Sync, context)?;

        // 11. Let next be not-started.
        // 12. Repeat, while next is not done,
        // 12.a. Set next to Completion(IteratorStepValue(inputIter)).
        // 12.b. IfAbruptCloseIterators(next, iters).
        while let Some(next) =
            if_abrupt_close_iterators!(input_iter.step_value(context), iters.iter(), context)
        {
            // 12.c. If next is not done, then
            // 12.c.i. Let iter be Completion(GetIteratorFlattenable(next, reject-primitives)).
            // 12.c.ii. IfAbruptCloseIterators(iter, the list-concatenation of « inputIter » and iters).
            // 12.c.iii. Append iter to iters.
            let iter = if_abrupt_close_iterators!(
                get_iterator_flattenable(&next, false, context),
                std::iter::once(&input_iter).chain(iters.iter()),
                context
            );
            iters.push(iter);
        }

        // 13 - 14 in build_padding_zip
        let padding = build_padding_zip(mode, padding_option, &iters, context)?;

        // 15. Let finishResults be a new Abstract Closure ... (handled in ZipIterator::create_zip_iterator)
        // 16. Return ? IteratorZip(iters, mode, padding, finishResults).
        let helper = IteratorHelper::create(
            iterator_helper::Zip::new(iters, mode, padding, ZipResultKind::Array),
            context,
        );
        Ok(helper.into())
    }

    /// `Iterator.zipKeyed ( iterables [ , options ] )`
    ///
    /// More information:
    ///  - [TC39 proposal][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iterator.zipkeyed
    fn zip_keyed(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterables = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. If iterables is not an Object, throw a TypeError exception.
        let iterables = iterables
            .as_object()
            .ok_or_else(|| js_error!(TypeError: "Iterator.zip requires an iterable object"))?;

        // 2 - 7 in options.
        let (mode, padding_option) = parse_options(options, context)?;

        // 8. Let iters be a new empty List.
        let mut iters: Vec<super::IteratorRecord> = Vec::new();

        // 9. Let padding be a new empty List.
        // (padding list built later in build_padding)

        // 10. Let allKeys be ? iterables.[[OwnPropertyKeys]]().
        let all_keys = iterables.own_property_keys(context)?;

        // 11. Let keys be a new empty List.
        let mut keys = Vec::new();
        // 12. For each element key of allKeys, do
        for key in all_keys {
            // 12.a. Let desc be Completion(iterables.[[GetOwnProperty]](key)).
            // 12.b. IfAbruptCloseIterators(desc, iters).
            let Some(desc) = if_abrupt_close_iterators!(
                iterables.__get_own_property__(&key, &mut context.into()),
                iters.iter(),
                context
            ) else {
                continue;
            };

            // 12.c. If desc is not undefined and desc.[[Enumerable]] is true, then
            if desc.enumerable() != Some(true) {
                continue;
            }

            // 12.c.i. Let value be Completion(Get(iterables, key)).
            // 12.c.ii. IfAbruptCloseIterators(value, iters).
            // 12.c.iii. If value is not undefined, then
            let value = if_abrupt_close_iterators!(
                iterables.get(key.clone(), context),
                iters.iter(),
                context
            );
            if value.is_undefined() {
                continue;
            }

            // 12.c.iii.1. Append key to keys.
            keys.push(key);

            // 12.c.iii.2. Let iter be Completion(GetIteratorFlattenable(value, reject-primitives)).
            // 12.c.iii.3. IfAbruptCloseIterators(iter, iters).
            let iter = if_abrupt_close_iterators!(
                get_iterator_flattenable(&value, false, context),
                iters.iter(),
                context
            );

            // 12.c.iii.4. Append iter to iters.
            iters.push(iter);
        }

        // 13 - 14 in build_padding_zip_keyed
        let padding = build_padding_zip_keyed(mode, padding_option, &iters, &keys, context)?;

        // 15. Let finishResults be a new Abstract Closure with parameters (results) that
        //     captures keys and iterCount and performs the following steps when called:
        // 15.a. Let obj be OrdinaryObjectCreate(null).
        // 15.b. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
        // 15.b.i. Perform ! CreateDataPropertyOrThrow(obj, keys[i], results[i]).
        // 15.b.c. Return obj.
        // All this is done within `Zip`.
        let helper = IteratorHelper::create(
            iterator_helper::Zip::new(iters, mode, padding, ZipResultKind::Keyed(keys)),
            context,
        );

        // 16. Return IteratorZip(iters, mode, padding, finishResults).
        Ok(helper.into())
    }
}

/// Parses the `mode` option from the options object.
fn parse_options(
    options: &JsValue,
    context: &mut Context,
) -> JsResult<(ZipMode, Option<JsObject>)> {
    // 2. Set options to ? GetOptionsObject(options).
    let options = get_options_object(options)?;

    // 3. Let mode be ? Get(options, "mode").
    let mode = options.get(js_string!("mode"), context)?;
    let mode = match mode.variant() {
        // 4. If mode is undefined, set mode to "shortest".
        JsVariant::Undefined => ZipMode::Shortest,
        JsVariant::String(mode) if mode == "shortest" => ZipMode::Shortest,
        JsVariant::String(mode) if mode == "longest" => ZipMode::Longest,
        JsVariant::String(mode) if mode == "strict" => ZipMode::Strict,
        // 5. If mode is not one of "shortest", "longest", or "strict", throw a TypeError exception.
        _ => {
            return Err(js_error!(TypeError: r#"mode must be "shortest", "longest", or "strict""#));
        }
    };

    // 6. Let paddingOption be undefined.
    // 7. If mode is "longest", then
    // 7.a. Set paddingOption to ? Get(options, "padding").
    // 7.b. If paddingOption is not undefined and paddingOption is not an Object, throw a TypeError exception.
    let padding_option = if mode == ZipMode::Longest {
        match options.get(js_string!("padding"), context)?.variant() {
            JsVariant::Undefined => None,
            JsVariant::Object(o) => Some(o),
            _ => return Err(js_error!(TypeError: "padding must be an object")),
        }
    } else {
        None
    };

    Ok((mode, padding_option))
}

/// Builds the padding list for "longest" mode on zip
fn build_padding_zip(
    mode: ZipMode,
    padding_option: Option<JsObject>,
    iters: &[super::IteratorRecord],
    context: &mut Context,
) -> JsResult<Vec<JsValue>> {
    // 13. Let iterCount be the number of elements in iters.
    let iter_count = iters.len();
    let padding_option = match padding_option {
        // 14. If mode is "longest", then
        Some(pad) if mode == ZipMode::Longest => pad,
        // 14.a. If paddingOption is undefined, then
        None if mode == ZipMode::Longest => {
            // 14.a.i. Perform the following steps iterCount times:
            // 14.a.i.1. Append undefined to padding.
            return Ok(vec![JsValue::undefined(); iter_count]);
        }
        _ => return Ok(Vec::new()),
    };

    // 14.b. Else,
    // 14.b.i. Let paddingIter be Completion(GetIterator(paddingOption, sync)).
    // 14.b.ii. IfAbruptCloseIterators(paddingIter, iters).
    let mut padding_iter = if_abrupt_close_iterators!(
        padding_option.get_iterator(IteratorHint::Sync, context),
        iters.iter(),
        context
    );
    let mut padding = Vec::new();
    // 14.b.iii. Let usingIterator be true.
    let mut using_iterator = true;

    // 14.b.iv. Perform the following steps iterCount times:
    for _ in 0..iter_count {
        // 14.b.iv.2. If usingIterator is false, append undefined to padding.
        if !using_iterator {
            padding.push(JsValue::undefined());
            continue;
        }

        // 14.b.iv.1. If usingIterator is true, then
        // 14.b.iv.1.a. Set next to Completion(IteratorStepValue(paddingIter)).
        // 14.b.iv.1.b. IfAbruptCloseIterators(next, iters).
        if let Some(next) =
            if_abrupt_close_iterators!(padding_iter.step_value(context), iters.iter(), context)
        {
            // 14.b.iv.1.d. Else,
            // 14.b.iv.1.d.i. Append next to padding.
            padding.push(next);
        } else {
            // 14.b.iv.1.c. If next is done, then
            // 14.b.iv.1.c.i. Set usingIterator to false.
            using_iterator = false;
            // 14.b.iv.2. If usingIterator is false, append undefined to padding.
            padding.push(JsValue::undefined());
        }
    }

    // 14.b.iv.2.v. If usingIterator is true, then
    if using_iterator {
        // 14.b.iv.2.v.1. Let completion be Completion(IteratorClose(paddingIter, NormalCompletion(unused))).
        // 14.b.iv.2.v.2. IfAbruptCloseIterators(completion, iters).
        if_abrupt_close_iterators!(
            padding_iter.close(Ok(JsValue::undefined()), context),
            iters.iter(),
            context
        );
    }

    Ok(padding)
}

/// Builds the padding list for "longest" mode on zipKeyed
fn build_padding_zip_keyed(
    mode: ZipMode,
    padding_option: Option<JsObject>,
    iters: &[super::IteratorRecord],
    keys: &[PropertyKey],
    context: &mut Context,
) -> JsResult<Vec<JsValue>> {
    // 13. Let iterCount be the number of elements in iters.
    let iter_count = iters.len();
    let padding_option = match padding_option {
        // 14. If mode is "longest", then
        Some(pad) if mode == ZipMode::Longest => pad,
        // 14.a. If paddingOption is undefined, then
        None if mode == ZipMode::Longest => {
            // 14.a.i. Perform the following steps iterCount times:
            // 14.a.i.1. Append undefined to padding.
            return Ok(vec![JsValue::undefined(); iter_count]);
        }
        _ => return Ok(Vec::new()),
    };

    let mut padding = Vec::new();

    // 14.b.i. For each element key of keys, do
    for key in keys {
        // 14.b.i.1. Let value be Completion(Get(paddingOption, key)).
        // 14.b.i.2. IfAbruptCloseIterators(value, iters).
        let value = if_abrupt_close_iterators!(
            padding_option.get(key.clone(), context),
            iters.iter(),
            context
        );

        // 14.b.i.3. Append value to padding.
        padding.push(value);
    }

    Ok(padding)
}
