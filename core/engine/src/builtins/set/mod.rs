//! Boa's implementation of ECMAScript's global `Set` object.
//!
//! The ECMAScript `Set` class is a global object that is used in the construction of sets; which
//! are high-level, collections of values.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-set-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set

mod set_iterator;

#[cfg(test)]
mod tests;

pub mod ordered_set;

use self::ordered_set::OrderedSet;
use super::iterable::IteratorHint;
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_engine::value::IntegerOrInfinity;
use boa_profiler::Profiler;
use num_traits::Zero;
pub(crate) use set_iterator::SetIterator;

/// A record containing information about a Set-like object.
#[derive(Debug)]
struct SetRecord {
    /// The Set-like object.
    object: JsObject,
    /// The size of the Set-like object.
    size: usize,
    /// The `has` method of the Set-like object.
    has: JsObject,
    /// The `keys` method of the Set-like object.
    keys: JsObject,
}

/// Implementation of the abstract operation `GetSetRecord`.
///
/// More information:
/// - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-getsetrecord
fn get_set_record(obj: &JsValue, context: &mut Context) -> JsResult<SetRecord> {
    // 1. If obj is not an Object, throw a TypeError exception.
    let obj = obj.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Set operation called with non-object argument")
    })?;

    // 2. Let rawSize be ? Get(obj, "size").
    let raw_size = obj.get(js_string!("size"), context)?;

    // 3. Let numSize be ? ToNumber(rawSize).
    // 4. NOTE: If rawSize is undefined, then numSize will be NaN.
    let num_size = raw_size.to_number(context)?;

    // 5. If numSize is NaN, throw a TypeError exception.
    if num_size.is_nan() {
        return Err(JsNativeError::typ()
            .with_message("size is undefined")
            .into());
    }

    // 6. Let intSize be ! ToIntegerOrInfinity(numSize).
    let int_size = IntegerOrInfinity::from(num_size);
    // 7. If intSize < 0, throw a RangeError exception.
    let size: usize = match int_size {
        IntegerOrInfinity::NegativeInfinity => {
            return Err(JsNativeError::range()
                .with_message("Set size must be non-negative")
                .into());
        }
        IntegerOrInfinity::Integer(size) if size < 0 => {
            return Err(JsNativeError::range()
                .with_message("Set size must be non-negative")
                .into());
        }
        IntegerOrInfinity::Integer(size) => size as usize,
        IntegerOrInfinity::PositiveInfinity => usize::MAX,
    };

    // 8. Let has be ? Get(obj, "has").
    let has = obj.get(js_string!("has"), context)?;

    // 9. If IsCallable(has) is false, throw a TypeError exception.
    let has = has.as_callable().ok_or_else(|| {
        JsNativeError::typ().with_message("Set-like object must have a callable 'has' method")
    })?;

    // 10. Let keys be ? Get(obj, "keys").
    let keys = obj.get(js_string!("keys"), context)?;

    // 11. If IsCallable(keys) is false, throw a TypeError exception.
    let keys = keys.as_callable().ok_or_else(|| {
        JsNativeError::typ().with_message("Set-like object must have a callable 'keys' method")
    })?;

    // 12. Return a new Set Record { [[SetObject]]: obj, [[Size]]: intSize, [[Has]]: has, [[Keys]]: keys }.
    Ok(SetRecord {
        object: obj.clone(),
        size,
        has: has.clone(),
        keys: keys.clone(),
    })
}

#[derive(Debug, Clone)]
pub(crate) struct Set;

impl IntrinsicObject for Set {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build();

        let size_getter = BuiltInBuilder::callable(realm, Self::size_getter)
            .name(js_string!("get size"))
            .build();

        let values_function = BuiltInBuilder::callable(realm, Self::values)
            .name(js_string!("values"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::add, js_string!("add"), 1)
            .method(Self::clear, js_string!("clear"), 0)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::entries, js_string!("entries"), 0)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::has, js_string!("has"), 1)
            .method(Self::difference, js_string!("difference"), 1)
            .method(Self::intersection, js_string!("intersection"), 1)
            .method(Self::is_disjoint_from, js_string!("isDisjointFrom"), 0)
            .method(Self::is_subset_of, js_string!("isSubsetOf"), 0)
            .method(Self::is_superset_of, js_string!("isSupersetOf"), 0)
            .method(
                Self::symmetric_difference,
                js_string!("symmetricDifference"),
                1,
            )
            .method(Self::union, js_string!("union"), 1)
            .property(
                js_string!("keys"),
                values_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("size"),
                Some(size_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("values"),
                values_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::iterator(),
                values_function,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }
}

impl BuiltInObject for Set {
    const NAME: JsString = StaticJsStrings::SET;
}

impl BuiltInConstructor for Set {
    const LENGTH: usize = 0;
    const P: usize = 11;
    const SP: usize = 1;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::set;

    /// [`Set ( [ iterable ] )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-iterable
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling a builtin Set constructor without new is forbidden")
                .into());
        }

        // 2. Let set be ? OrdinaryCreateFromConstructor(NewTarget, "%Set.prototype%", « [[SetData]] »).
        // 3. Set set.[[SetData]] to a new empty List.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::set, context)?;
        let set = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrderedSet::default(),
        );

        // 4. If iterable is either undefined or null, return set.
        let iterable = args.get_or_undefined(0);
        if iterable.is_null_or_undefined() {
            return Ok(set.into());
        }

        // 5. Let adder be ? Get(set, "add").
        let adder = set.get(js_string!("add"), context)?;

        // 6. If IsCallable(adder) is false, throw a TypeError exception.
        let adder = adder.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("'add' of 'newTarget' is not a function")
        })?;

        // 7. Let iteratorRecord be ? GetIterator(iterable, sync).
        let mut iterator_record = iterable.clone().get_iterator(IteratorHint::Sync, context)?;

        // 8. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(next) = iterator_record.step_value(context)? {
            // c. Let status be Completion(Call(adder, set, « next »)).
            if let Err(status) = adder.call(&set.clone().into(), &[next], context) {
                // d. IfAbruptCloseIterator(status, iteratorRecord).
                return iterator_record.close(Err(status), context);
            }
        }

        //     b. If next is done, return set.
        Ok(set.into())
    }
}

impl Set {
    /// Utility for constructing `Set` objects.
    pub(crate) fn set_create(prototype: Option<JsObject>, context: &mut Context) -> JsObject {
        let prototype =
            prototype.unwrap_or_else(|| context.intrinsics().constructors().set().prototype());

        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrderedSet::new(),
        )
    }

    /// Utility for constructing `Set` objects from an iterator of `JsValue`'s.
    pub(crate) fn create_set_from_list<I>(elements: I, context: &mut Context) -> JsObject
    where
        I: IntoIterator<Item = JsValue>,
    {
        // Create empty Set
        let set = Self::set_create(None, context);
        // For each element e of elements, do
        for elem in elements {
            Self::add(&set.clone().into(), &[elem], context)
                .expect("adding new element shouldn't error out");
        }

        set
    }

    /// `get Set [ @@species ]`
    ///
    /// The Set[Symbol.species] accessor property returns the Set constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-set-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `Set.prototype.add( value )`
    ///
    /// This method adds an entry with value into the set. Returns the set object
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.add
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/add
    pub(crate) fn add(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(mut set) = this
            .as_object()
            .and_then(JsObject::downcast_mut::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.add called on incompatible receiver")
                .into());
        };

        // 3. For each element e of S.[[SetData]], do
        // a. If e is not empty and SameValueZero(e, value) is true, then
        // i. Return S.
        // 4. If value is -0𝔽, set value to +0𝔽.
        let value = args.get_or_undefined(0);
        let value = match value.as_number() {
            Some(n) if n.is_zero() => &JsValue::new(0),
            _ => value,
        };

        // 5. Append value to S.[[SetData]].
        set.add(value.clone());

        Ok(this.clone())
        // 6. Return S.
    }

    /// `Set.prototype.clear( )`
    ///
    /// This method removes all entries from the set.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.clear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/clear
    pub(crate) fn clear(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let Some(mut set) = this
            .as_object()
            .and_then(JsObject::downcast_mut::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("'this' is not a Set")
                .into());
        };

        set.clear();

        Ok(JsValue::undefined())
    }

    /// `Set.prototype.delete( value )`
    ///
    /// This method removes the entry for the given value if it exists.
    /// Returns true if there was an element, false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/delete
    pub(crate) fn delete(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(mut set) = this
            .as_object()
            .and_then(JsObject::downcast_mut::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.delete called on incompatible receiver")
                .into());
        };

        let value = args.get_or_undefined(0);
        let value = match value.as_number() {
            Some(n) if n.is_zero() => &JsValue::new(0),
            _ => value,
        };

        // 3. For each element e of S.[[SetData]], do
        // a. If e is not empty and SameValueZero(e, value) is true, then
        // i. Replace the element of S.[[SetData]] whose value is e with an element whose value is empty.
        // ii. Return true.
        // 4. Return false.
        Ok(set.delete(value).into())
    }

    /// `Set.prototype.entries( )`
    ///
    /// This method returns an iterator over the entries of the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.entries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/entries
    pub(crate) fn entries(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.entries called on incompatible receiver")
                .into());
        };

        Ok(SetIterator::create_set_iterator(
            this.clone(),
            PropertyNameKind::KeyAndValue,
            lock,
            context,
        ))
    }

    /// `Set.prototype.forEach( callbackFn [ , thisArg ] )`
    ///
    /// This method executes the provided callback function for each value in the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.foreach
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/foreach
    pub(crate) fn for_each(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.forEach called on incompatible receiver")
                .into());
        };

        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let Some(callback_fn) = args.get_or_undefined(0).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message(
                    "Method Set.prototype.forEach called with non-callable callback function",
                )
                .into());
        };

        // 4. Let entries be S.[[SetData]].
        // 5. Let numEntries be the number of elements in entries.
        // 6. Let index be 0.
        let mut index = 0;

        // 7. Repeat, while index < numEntries,
        while index < Self::get_size_full(this)? {
            // a. Let e be entries[index].
            let Some(set) = this
                .as_object()
                .and_then(JsObject::downcast_ref::<OrderedSet>)
            else {
                return Err(JsNativeError::typ()
                    .with_message("Method Set.prototype.forEach called on incompatible receiver")
                    .into());
            };

            let e = set.get_index(index).cloned();
            drop(set);

            // b. Set index to index + 1.
            index += 1;

            // c. If e is not empty, then
            if let Some(e) = e {
                // i. Perform ? Call(callbackfn, thisArg, « e, e, S »).
                // ii. NOTE: The number of elements in entries may have increased during execution of callbackfn.
                // iii. Set numEntries to the number of elements in entries.
                callback_fn.call(
                    args.get_or_undefined(1),
                    &[e.clone(), e.clone(), this.clone()],
                    context,
                )?;
            }
        }

        drop(lock);

        // 8. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `Map.prototype.has( key )`
    ///
    /// This method checks if the map contains an entry with the given key.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has
    pub(crate) fn has(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Perform ? RequireInternalSlot(S, [[SetData]]).
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.has called on incompatible receiver")
                .into());
        };

        let value = args.get_or_undefined(0);
        let value = match value.as_number() {
            Some(n) if n.is_zero() => &JsValue::new(0),
            _ => value,
        };

        // 3. For each element e of S.[[SetData]], do
        // a. If e is not empty and SameValueZero(e, value) is true, return true.
        // 4. Return false.
        Ok(set.contains(value).into())
    }

    /// `Set.prototype.values( )`
    ///
    /// This method returns an iterator over the values of the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/values
    pub(crate) fn values(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.values called on incompatible receiver")
                .into());
        };

        Ok(SetIterator::create_set_iterator(
            this.clone(),
            PropertyNameKind::Value,
            lock,
            context,
        ))
    }

    /// `Set.prototype.isDisjointFrom ( other )`
    ///
    /// This method checks whether the current Set and the given iterable `other` have no elements in common.
    /// It returns `true` if the two Sets are disjoint (i.e., they have no overlapping elements),
    /// and `false` otherwise.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.isdisjointfrom
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/isDisjointFrom
    pub(crate) fn is_disjoint_from(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isDisjointFrom called on incompatible receiver")
                .into());
        };

        // 2. Let otherRec be ? GetSetRecord(other).
        let other_rec = get_set_record(args.get_or_undefined(0), context)?;

        // 3. If SetDataSize(O.[[SetData]]) > otherRec.[[Size]], return false.
        if Self::get_size_full(this)? <= other_rec.size {
            for value in set.iter() {
                // Use the has method from the SetRecord
                let has_result = other_rec.has.call(
                    &other_rec.object.clone().into(),
                    &[value.clone()],
                    context,
                )?;
                if has_result.to_boolean() {
                    return Ok(JsValue::from(false));
                }
            }
        } else {
            // Get an iterator from the other set's keys method
            let keys_result =
                other_rec
                    .keys
                    .call(&other_rec.object.clone().into(), &[], context)?;
            let mut iterator_record = keys_result.get_iterator(IteratorHint::Sync, context)?;

            while let Some(value) = iterator_record.step_value(context)? {
                if set.contains(&value) {
                    return Ok(JsValue::from(false));
                }
            }
        }

        // 4. Return true if no common elements are found.
        Ok(JsValue::from(true))
    }

    /// `Set.prototype.isSubsetOf ( other )`
    ///
    /// This method checks whether the current Set is a subset of the given iterable `other`.
    /// It returns `true` if all elements of the current Set are present in the given iterable,
    /// and `false` otherwise.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.issubsetof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/isSubsetOf
    pub(crate) fn is_subset_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isSubsetOf called on incompatible receiver")
                .into());
        };

        // 2. Let otherRec be ? GetSetRecord(other).
        let other_rec = get_set_record(args.get_or_undefined(0), context)?;

        // 3. If SetDataSize(O.[[SetData]]) > otherRec.[[Size]], return false.
        if Self::get_size_full(this)? > other_rec.size {
            return Ok(JsValue::from(false));
        }

        // 4. For each element in the current set
        for value in set.iter() {
            // 5. Check if the element exists in otherRec using its has method
            let has_result =
                other_rec
                    .has
                    .call(&other_rec.object.clone().into(), &[value.clone()], context)?;
            if !has_result.to_boolean() {
                return Ok(JsValue::from(false));
            }
        }

        // 6. Return true if all elements were found
        Ok(JsValue::from(true))
    }

    /// `Set.prototype.isSupersetOf ( other )`
    ///
    /// This method checks whether the current Set is a superset of the given iterable `other`.
    /// It returns `true` if the current Set contains all elements from the given iterable,
    /// and `false` otherwise.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.issupersetof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/isSupersetOf
    pub(crate) fn is_superset_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.isSupersetOf called on incompatible receiver")
                .into());
        };

        // 2. Let otherRec be ? GetSetRecord(other).
        let other_rec = get_set_record(args.get_or_undefined(0), context)?;

        // 3. If SetDataSize(O.[[SetData]]) < otherRec.[[Size]], return false.
        if Self::get_size_full(this)? < other_rec.size {
            return Ok(JsValue::from(false));
        }

        // 4. Get an iterator from the other set's keys method
        let keys_result = other_rec
            .keys
            .call(&other_rec.object.into(), &[], context)?;
        let mut iterator_record = keys_result.get_iterator(IteratorHint::Sync, context)?;

        // 5. Check each element from other set
        while let Some(value) = iterator_record.step_value(context)? {
            if !set.contains(&value) {
                return Ok(JsValue::from(false));
            }
        }

        // 6. Return true if all elements were found
        Ok(JsValue::from(true))
    }

    /// ` Set.prototype.symmetricDifference(other)`
    ///
    /// Returns a new set containing the symmetric difference between the current set (`this`)
    /// and the provided set (`other`)
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.symmerticDifference
    /// [mdn]: https://developer.mozilla.org/en-USSet/docs/Web/JavaScript/Reference/Global_Objects/Set/symmetricDifference
    pub(crate) fn symmetric_difference(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[SetData]]).
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message(
                    "Method Set.prototype.symmetricDifference called on incompatible receiver",
                )
                .into());
        };

        // 3. Let otherRec be ? GetSetRecord(other).
        let other_rec = get_set_record(args.get_or_undefined(0), context)?;

        // 4. Let keysIter be ? GetIteratorFromMethod(otherRec.[[SetObject]], otherRec.[[Keys]]).
        let keys_result = other_rec
            .keys
            .call(&other_rec.object.clone().into(), &[], context)?;
        let mut iterator_record = keys_result.get_iterator(IteratorHint::Sync, context)?;

        // 5. Let resultSetData be a copy of O.[[SetData]].
        let mut result_set = set.clone();

        // 6. Let next be not-started.
        // 7. Repeat, while next is not done,
        while let Some(next_value) = iterator_record.step_value(context)? {
            // a. Set next to ? IteratorStepValue(keysIter).
            // b. If next is not done, then
            // i. Set next to CanonicalizeKeyedCollectionKey(next).
            let next = match next_value.as_number() {
                Some(n) if n.is_zero() => JsValue::new(0),
                _ => next_value,
            };

            // ii. Let resultIndex be SetDataIndex(resultSetData, next).
            // iii. If resultIndex is not-found, let alreadyInResult be false.
            // Otherwise let alreadyInResult be true.
            let already_in_result = result_set.contains(&next);

            // iv. If SetDataHas(O.[[SetData]], next) is true, then
            if set.contains(&next) {
                // 1. If alreadyInResult is true, set resultSetData[resultIndex] to empty.
                if already_in_result {
                    result_set.delete(&next);
                }
            } else {
                // v. Else,
                // 1. If alreadyInResult is false, append next to resultSetData.
                if !already_in_result {
                    result_set.add(next);
                }
            }
        }

        // 8. Let result be OrdinaryObjectCreate(%Set.prototype%, « [[SetData]] »).
        // 9. Set result.[[SetData]] to resultSetData.
        // 10. Return result.
        Ok(Self::create_set_from_list(result_set.iter().cloned(), context).into())
    }

    /// `Set.prototype.union ( other )`
    ///
    /// Returns a new set containing the union of the elements in the current set (`this`)
    /// and the set provided as the argument (`other`).
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.union
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/union
    pub(crate) fn union(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let Some(lock) = this.as_object().and_then(|o| {
            o.downcast_mut::<OrderedSet>()
                .map(|mut set| set.lock(o.clone()))
        }) else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.entries called on incompatible receiver")
                .into());
        };

        // 3. Let otherRec be ? GetSetRecord(other).
        let other = args.get_or_undefined(0);
        let other_rec = get_set_record(other, context)?;

        // 4. Let keysIter be ? GetIteratorFromMethod(otherRec.[[SetObject]], otherRec.[[Keys]]).
        let mut iterator_record = other.get_iterator_from_method(&other_rec.keys, context)?;

        // 5. Let resultSetData be a copy of O.[[SetData]].
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.union called on incompatible receiver")
                .into());
        };
        let mut result_set = OrderedSet::clone(&set);
        drop(set); // Make sure we don't retain a borrow on this.

        // 6. Let next be not-started.
        // 7. Repeat, while next is not done,
        //        a. Set next to ? IteratorStepValue(keysIter).
        //        b. If next is not done, then
        //               i. Set next to CanonicalizeKeyedCollectionKey(next).
        //               ii. If SetDataHas(resultSetData, next) is false, then
        //                       1. Append next to resultSetData.
        while let Some(value) = iterator_record.step_value(context)? {
            result_set.add(value.clone());
        }

        drop(lock);

        // 8. Let result be OrdinaryObjectCreate(%Set.prototype%, « [[SetData]] »).
        // 9. Set result.[[SetData]] to resultSetData.
        // 10. Return result.
        Ok(Self::create_set_from_list(result_set.iter().cloned(), context).into())
    }

    /// `Set.prototype.intersection ( other )`
    ///
    /// This method returns a new Set containing all elements that are present in both
    /// the current Set and the given iterable `other`.
    ///
    /// It effectively computes the intersection of the two Sets.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.intersection
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/intersection
    pub(crate) fn intersection(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.intersection called on incompatible receiver")
                .into());
        };

        // 2. Let otherRec be ? GetSetRecord(other).
        let other_rec = get_set_record(args.get_or_undefined(0), context)?;

        // Create an empty result set.
        let mut result_set = OrderedSet::new();

        // Optimize by iterating over the smaller set
        if Self::get_size_full(this)? <= other_rec.size {
            for value in set.iter() {
                // Check if the element exists in otherRec using its has method
                let has_result = other_rec.has.call(
                    &other_rec.object.clone().into(),
                    &[value.clone()],
                    context,
                )?;
                if has_result.to_boolean() {
                    result_set.add(value.clone());
                }
            }
        } else {
            // Get an iterator from the other set's keys method
            let keys_result =
                other_rec
                    .keys
                    .call(&other_rec.object.clone().into(), &[], context)?;
            let mut iterator_record = keys_result.get_iterator(IteratorHint::Sync, context)?;

            while let Some(value) = iterator_record.step_value(context)? {
                if set.contains(&value) {
                    result_set.add(value);
                }
            }
        }

        // Return the result set.
        Ok(Set::create_set_from_list(result_set.iter().cloned(), context).into())
    }

    /// ` Set.prototype.difference ( other ) `
    ///
    /// This method returns a new Set containing all elements that are in the current Set
    /// but not in the given iterable `other`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.difference
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/difference
    pub(crate) fn difference(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let Some(set) = this
            .as_object()
            .and_then(JsObject::downcast_ref::<OrderedSet>)
        else {
            return Err(JsNativeError::typ()
                .with_message("Method Set.prototype.difference called on incompatible receiver")
                .into());
        };

        // 2. Let otherRec be ? GetSetRecord(other).
        let other_rec = get_set_record(args.get_or_undefined(0), context)?;

        // 3. Let resultSetData be a copy of O.[[SetData]].
        let mut result_set = set.clone();

        // 4. If SetDataSize(O.[[SetData]]) ≤ otherRec.[[Size]], then:
        if Self::get_size_full(this)? <= other_rec.size {
            // Iterate over elements of the current set.
            let elements: Vec<_> = result_set.iter().cloned().collect();
            for element in elements {
                // Check if the element exists in otherRec using its has method
                let has_result = other_rec.has.call(
                    &other_rec.object.clone().into(),
                    &[element.clone()],
                    context,
                )?;
                if has_result.to_boolean() {
                    result_set.delete(&element);
                }
            }
        } else {
            // Get an iterator from the other set's keys method
            let keys_result =
                other_rec
                    .keys
                    .call(&other_rec.object.clone().into(), &[], context)?;
            let mut iterator_record = keys_result.get_iterator(IteratorHint::Sync, context)?;

            while let Some(element) = iterator_record.step_value(context)? {
                result_set.delete(&element);
            }
        }

        // 5. Return a new set with the updated resultSetData.
        Ok(Self::create_set_from_list(result_set.iter().cloned(), context).into())
    }

    fn size_getter(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Self::get_size(this).map(JsValue::from)
    }

    /// Helper function to get the size of the `Set` object.
    pub(crate) fn get_size(set: &JsValue) -> JsResult<usize> {
        set.as_object()
            .and_then(|obj| {
                obj.borrow()
                    .downcast_ref::<OrderedSet>()
                    .map(OrderedSet::len)
            })
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Set")
                    .into()
            })
    }

    /// Helper function to get the full size of the `Set` object.
    pub(crate) fn get_size_full(set: &JsValue) -> JsResult<usize> {
        set.as_object()
            .and_then(|obj| {
                obj.borrow()
                    .downcast_ref::<OrderedSet>()
                    .map(OrderedSet::full_len)
            })
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Set")
                    .into()
            })
    }
}
