//! Boa's implementation of ECMAScript's global `JSON` object.
//!
//! The `JSON` object contains methods for parsing [JavaScript Object Notation (JSON)][spec]
//! and converting values to JSON. It can't be called or constructed, and aside from its
//! two method properties, it has no interesting functionality of its own.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!  - [JSON specification][json]
//!
//! [spec]: https://tc39.es/ecma262/#sec-json
//! [json]: https://www.json.org/json-en.html
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON

use std::{borrow::Cow, iter::once};

use itertools::Itertools;

use crate::{
    builtins::BuiltInObject,
    bytecompiler::ByteCompiler,
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    string::{utf16, CodePoint},
    symbol::JsSymbol,
    value::IntegerOrInfinity,
    vm::CallFrame,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_gc::Gc;
use boa_interner::Sym;
use boa_parser::{Parser, Source};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, IntrinsicObject};

#[cfg(test)]
mod tests;

/// JavaScript `JSON` global object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Json;

impl IntrinsicObject for Json {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let to_string_tag = JsSymbol::to_string_tag();
        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(Self::parse, "parse", 2)
            .static_method(Self::stringify, "stringify", 3)
            .static_property(to_string_tag, Self::NAME, attribute)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().json()
    }
}

impl BuiltInObject for Json {
    const NAME: &'static str = "JSON";
}

impl Json {
    /// `JSON.parse( text[, reviver] )`
    ///
    /// This `JSON` method parses a JSON string, constructing the JavaScript value or object described by the string.
    ///
    /// An optional `reviver` function can be provided to perform a transformation on the resulting object before it is returned.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-json.parse
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
    pub(crate) fn parse(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let jsonString be ? ToString(text).
        let json_string = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?
            .to_std_string()
            .map_err(|e| JsNativeError::syntax().with_message(e.to_string()))?;

        // 2. Parse ! StringToCodePoints(jsonString) as a JSON text as specified in ECMA-404.
        //    Throw a SyntaxError exception if it is not a valid JSON text as defined in that specification.
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&json_string) {
            return Err(JsNativeError::syntax().with_message(e.to_string()).into());
        }

        // 3. Let scriptString be the string-concatenation of "(", jsonString, and ");".
        // TODO: fix script read for eval
        let script_string = format!("({json_string});");

        // 4. Let script be ParseText(! StringToCodePoints(scriptString), Script).
        // 5. NOTE: The early error rules defined in 13.2.5.1 have special handling for the above invocation of ParseText.
        // 6. Assert: script is a Parse Node.
        // 7. Let completion be the result of evaluating script.
        // 8. NOTE: The PropertyDefinitionEvaluation semantics defined in 13.2.5.5 have special handling for the above evaluation.
        // 9. Let unfiltered be completion.[[Value]].
        // 10. Assert: unfiltered is either a String, Number, Boolean, Null, or an Object that is defined by either an ArrayLiteral or an ObjectLiteral.
        let mut parser = Parser::new(Source::from_bytes(&script_string));
        parser.set_json_parse();
        let script = parser.parse_script(context.interner_mut())?;
        let code_block = {
            let mut compiler = ByteCompiler::new(
                Sym::MAIN,
                script.strict(),
                true,
                context.realm().environment().compile_env(),
                context,
            );
            compiler.compile_statement_list(script.statements(), true, false);
            Gc::new(compiler.finish())
        };

        context.vm.push_frame(CallFrame::new(code_block));
        context.realm().resize_global_env();
        let record = context.run();
        context.vm.pop_frame();

        let unfiltered = record.consume()?;

        // 11. If IsCallable(reviver) is true, then
        if let Some(obj) = args.get_or_undefined(1).as_callable() {
            // a. Let root be ! OrdinaryObjectCreate(%Object.prototype%).
            let root = JsObject::with_object_proto(context.intrinsics());

            // b. Let rootName be the empty String.
            // c. Perform ! CreateDataPropertyOrThrow(root, rootName, unfiltered).
            root.create_data_property_or_throw(utf16!(""), unfiltered, context)
                .expect("CreateDataPropertyOrThrow should never throw here");

            // d. Return ? InternalizeJSONProperty(root, rootName, reviver).
            Self::internalize_json_property(&root, "".into(), obj, context)
        } else {
            // 12. Else,
            // a. Return unfiltered.
            Ok(unfiltered)
        }
    }

    /// `25.5.1.1 InternalizeJSONProperty ( holder, name, reviver )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-internalizejsonproperty
    fn internalize_json_property(
        holder: &JsObject,
        name: JsString,
        reviver: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let val be ? Get(holder, name).
        let val = holder.get(name.clone(), context)?;

        // 2. If Type(val) is Object, then
        if let Some(obj) = val.as_object() {
            // a. Let isArray be ? IsArray(val).
            // b. If isArray is true, then
            if obj.is_array_abstract()? {
                // i. Let I be 0.
                // ii. Let len be ? LengthOfArrayLike(val).
                // iii. Repeat, while I < len,
                let len = obj.length_of_array_like(context)? as i64;
                for i in 0..len {
                    // 1. Let prop be ! ToString(𝔽(I)).
                    // 2. Let newElement be ? InternalizeJSONProperty(val, prop, reviver).
                    let new_element = Self::internalize_json_property(
                        obj,
                        i.to_string().into(),
                        reviver,
                        context,
                    )?;

                    // 3. If newElement is undefined, then
                    if new_element.is_undefined() {
                        // a. Perform ? val.[[Delete]](prop).
                        obj.__delete__(&i.into(), context)?;
                    }
                    // 4. Else,
                    else {
                        // a. Perform ? CreateDataProperty(val, prop, newElement).
                        obj.create_data_property(i, new_element, context)?;
                    }
                }
            }
            // c. Else,
            else {
                // i. Let keys be ? EnumerableOwnPropertyNames(val, key).
                let keys = obj.enumerable_own_property_names(PropertyNameKind::Key, context)?;

                // ii. For each String P of keys, do
                for p in keys {
                    // This is safe, because EnumerableOwnPropertyNames with 'key' type only returns strings.
                    let p = p
                        .as_string()
                        .expect("EnumerableOwnPropertyNames only returns strings")
                        .clone();

                    // 1. Let newElement be ? InternalizeJSONProperty(val, P, reviver).
                    let new_element =
                        Self::internalize_json_property(obj, p.clone(), reviver, context)?;

                    // 2. If newElement is undefined, then
                    if new_element.is_undefined() {
                        // a. Perform ? val.[[Delete]](P).
                        obj.__delete__(&p.into(), context)?;
                    }
                    // 3. Else,
                    else {
                        // a. Perform ? CreateDataProperty(val, P, newElement).
                        obj.create_data_property(p, new_element, context)?;
                    }
                }
            }
        }

        // 3. Return ? Call(reviver, holder, « name, val »).
        reviver.call(&holder.clone().into(), &[name.into(), val], context)
    }

    /// `JSON.stringify( value[, replacer[, space]] )`
    ///
    /// This `JSON` method converts a JavaScript object or value to a JSON string.
    ///
    /// This method optionally replaces values if a `replacer` function is specified or
    /// optionally including only the specified properties if a replacer array is specified.
    ///
    /// An optional `space` argument can be supplied of type `String` or `Number` that's used to insert
    /// white space into the output JSON string for readability purposes.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-json.stringify
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify
    pub(crate) fn stringify(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let stack be a new empty List.
        let stack = Vec::new();

        // 2. Let indent be the empty String.
        let indent = js_string!();

        // 3. Let PropertyList and ReplacerFunction be undefined.
        let mut property_list = None;
        let mut replacer_function = None;

        let replacer = args.get_or_undefined(1);

        // 4. If Type(replacer) is Object, then
        if let Some(replacer_obj) = replacer.as_object() {
            // a. If IsCallable(replacer) is true, then
            if replacer_obj.is_callable() {
                // i. Set ReplacerFunction to replacer.
                replacer_function = Some(replacer_obj.clone());
            // b. Else,
            } else {
                // i. Let isArray be ? IsArray(replacer).
                // ii. If isArray is true, then
                if replacer_obj.is_array_abstract()? {
                    // 1. Set PropertyList to a new empty List.
                    let mut property_set = indexmap::IndexSet::new();

                    // 2. Let len be ? LengthOfArrayLike(replacer).
                    let len = replacer_obj.length_of_array_like(context)?;

                    // 3. Let k be 0.
                    let mut k = 0;

                    // 4. Repeat, while k < len,
                    while k < len {
                        // a. Let prop be ! ToString(𝔽(k)).
                        // b. Let v be ? Get(replacer, prop).
                        let v = replacer_obj.get(k, context)?;

                        // c. Let item be undefined.
                        // d. If Type(v) is String, set item to v.
                        // e. Else if Type(v) is Number, set item to ! ToString(v).
                        // f. Else if Type(v) is Object, then
                        // g. If item is not undefined and item is not currently an element of PropertyList, then
                        // i. Append item to the end of PropertyList.
                        if let Some(s) = v.as_string() {
                            property_set.insert(s.clone());
                        } else if v.is_number() {
                            property_set.insert(
                                v.to_string(context)
                                    .expect("ToString cannot fail on number value"),
                            );
                        } else if let Some(obj) = v.as_object() {
                            // i. If v has a [[StringData]] or [[NumberData]] internal slot, set item to ? ToString(v).
                            if obj.is_string() || obj.is_number() {
                                property_set.insert(v.to_string(context)?);
                            }
                        }

                        // h. Set k to k + 1.
                        k += 1;
                    }
                    property_list = Some(property_set.into_iter().collect());
                }
            }
        }

        let mut space = args.get_or_undefined(2).clone();

        // 5. If Type(space) is Object, then
        if let Some(space_obj) = space.as_object() {
            // a. If space has a [[NumberData]] internal slot, then
            if space_obj.is_number() {
                // i. Set space to ? ToNumber(space).
                space = space.to_number(context)?.into();
            }
            // b. Else if space has a [[StringData]] internal slot, then
            else if space_obj.is_string() {
                // i. Set space to ? ToString(space).
                space = space.to_string(context)?.into();
            }
        }

        // 6. If Type(space) is Number, then
        let gap = if space.is_number() {
            // a. Let spaceMV be ! ToIntegerOrInfinity(space).
            // b. Set spaceMV to min(10, spaceMV).
            // c. If spaceMV < 1, let gap be the empty String; otherwise let gap be the String value containing spaceMV occurrences of the code unit 0x0020 (SPACE).
            match space
                .to_integer_or_infinity(context)
                .expect("ToIntegerOrInfinity cannot fail on number")
            {
                IntegerOrInfinity::PositiveInfinity => js_string!("          "),
                IntegerOrInfinity::NegativeInfinity => js_string!(),
                IntegerOrInfinity::Integer(i) if i < 1 => js_string!(),
                IntegerOrInfinity::Integer(i) => {
                    let mut s = String::new();
                    let i = std::cmp::min(10, i);
                    for _ in 0..i {
                        s.push(' ');
                    }
                    s.into()
                }
            }
        // 7. Else if Type(space) is String, then
        } else if let Some(s) = space.as_string() {
            // a. If the length of space is 10 or less, let gap be space; otherwise let gap be the substring of space from 0 to 10.
            js_string!(s.get(..10).unwrap_or(s))
        // 8. Else,
        } else {
            // a. Let gap be the empty String.
            js_string!()
        };

        // 9. Let wrapper be ! OrdinaryObjectCreate(%Object.prototype%).
        let wrapper = JsObject::with_object_proto(context.intrinsics());

        // 10. Perform ! CreateDataPropertyOrThrow(wrapper, the empty String, value).
        wrapper
            .create_data_property_or_throw(utf16!(""), args.get_or_undefined(0).clone(), context)
            .expect("CreateDataPropertyOrThrow should never fail here");

        // 11. Let state be the Record { [[ReplacerFunction]]: ReplacerFunction, [[Stack]]: stack, [[Indent]]: indent, [[Gap]]: gap, [[PropertyList]]: PropertyList }.
        let mut state = StateRecord {
            replacer_function,
            stack,
            indent,
            gap,
            property_list,
        };

        // 12. Return ? SerializeJSONProperty(state, the empty String, wrapper).
        Ok(
            Self::serialize_json_property(&mut state, js_string!(), &wrapper, context)?
                .map(Into::into)
                .unwrap_or_default(),
        )
    }

    /// `25.5.2.1 SerializeJSONProperty ( state, key, holder )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-serializejsonproperty
    fn serialize_json_property(
        state: &mut StateRecord,
        key: JsString,
        holder: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<Option<JsString>> {
        // 1. Let value be ? Get(holder, key).
        let mut value = holder.get(key.clone(), context)?;

        // 2. If Type(value) is Object or BigInt, then
        if value.is_object() || value.is_bigint() {
            // a. Let toJSON be ? GetV(value, "toJSON").
            let to_json = value.get_v(utf16!("toJSON"), context)?;

            // b. If IsCallable(toJSON) is true, then
            if let Some(obj) = to_json.as_object() {
                if obj.is_callable() {
                    // i. Set value to ? Call(toJSON, value, « key »).
                    value = obj.call(&value, &[key.clone().into()], context)?;
                }
            }
        }

        // 3. If state.[[ReplacerFunction]] is not undefined, then
        if let Some(obj) = &state.replacer_function {
            // a. Set value to ? Call(state.[[ReplacerFunction]], holder, « key, value »).
            value = obj.call(&holder.clone().into(), &[key.into(), value], context)?;
        }

        // 4. If Type(value) is Object, then
        if let Some(obj) = value.as_object().cloned() {
            // a. If value has a [[NumberData]] internal slot, then
            if obj.is_number() {
                // i. Set value to ? ToNumber(value).
                value = value.to_number(context)?.into();
            }
            // b. Else if value has a [[StringData]] internal slot, then
            else if obj.is_string() {
                // i. Set value to ? ToString(value).
                value = value.to_string(context)?.into();
            }
            // c. Else if value has a [[BooleanData]] internal slot, then
            else if let Some(boolean) = obj.borrow().as_boolean() {
                // i. Set value to value.[[BooleanData]].
                value = boolean.into();
            }
            // d. Else if value has a [[BigIntData]] internal slot, then
            else if let Some(bigint) = obj.borrow().as_bigint() {
                // i. Set value to value.[[BigIntData]].
                value = bigint.clone().into();
            }
        }

        // 5. If value is null, return "null".
        if value.is_null() {
            return Ok(Some(js_string!("null")));
        }

        // 6. If value is true, return "true".
        // 7. If value is false, return "false".
        if value.is_boolean() {
            return Ok(Some(js_string!(if value.to_boolean() {
                "true"
            } else {
                "false"
            })));
        }

        // 8. If Type(value) is String, return QuoteJSONString(value).
        if let Some(s) = value.as_string() {
            return Ok(Some(Self::quote_json_string(s)));
        }

        // 9. If Type(value) is Number, then
        if let Some(n) = value.as_number() {
            // a. If value is finite, return ! ToString(value).
            if n.is_finite() {
                return Ok(Some(
                    value
                        .to_string(context)
                        .expect("ToString should never fail here"),
                ));
            }

            // b. Return "null".
            return Ok(Some(js_string!("null")));
        }

        // 10. If Type(value) is BigInt, throw a TypeError exception.
        if value.is_bigint() {
            return Err(JsNativeError::typ()
                .with_message("cannot serialize bigint to JSON")
                .into());
        }

        // 11. If Type(value) is Object and IsCallable(value) is false, then
        if let Some(obj) = value.as_object() {
            if !obj.is_callable() {
                // a. Let isArray be ? IsArray(value).
                // b. If isArray is true, return ? SerializeJSONArray(state, value).
                // c. Return ? SerializeJSONObject(state, value).
                return if obj.is_array_abstract()? {
                    Ok(Some(Self::serialize_json_array(state, obj, context)?))
                } else {
                    Ok(Some(Self::serialize_json_object(state, obj, context)?))
                };
            }
        }

        // 12. Return undefined.
        Ok(None)
    }

    /// `25.5.2.2 QuoteJSONString ( value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-quotejsonstring
    fn quote_json_string(value: &JsString) -> JsString {
        let mut buf = [0; 2];
        // 1. Let product be the String value consisting solely of the code unit 0x0022 (QUOTATION MARK).
        let mut product = vec!['"' as u16];

        // 2. For each code point C of ! StringToCodePoints(value), do
        for code_point in value.code_points() {
            match code_point {
                // a. If C is listed in the “Code Point” column of Table 73, then
                // i. Set product to the string-concatenation of product and the
                // escape sequence for C as specified in the “Escape Sequence”
                // column of the corresponding row.
                CodePoint::Unicode('\u{0008}') => product.extend_from_slice(utf16!(r"\b")),
                CodePoint::Unicode('\u{0009}') => product.extend_from_slice(utf16!(r"\t")),
                CodePoint::Unicode('\u{000A}') => product.extend_from_slice(utf16!(r"\n")),
                CodePoint::Unicode('\u{000C}') => product.extend_from_slice(utf16!(r"\f")),
                CodePoint::Unicode('\u{000D}') => product.extend_from_slice(utf16!(r"\r")),
                CodePoint::Unicode('\u{0022}') => product.extend_from_slice(utf16!(r#"\""#)),
                CodePoint::Unicode('\u{005C}') => product.extend_from_slice(utf16!(r"\\")),
                // b. Else if C has a numeric value less than 0x0020 (SPACE), or
                // if C has the same numeric value as a leading surrogate or
                // trailing surrogate, then
                //     i. Let unit be the code unit whose numeric value is that
                //     of C.
                //     ii. Set product to the string-concatenation of product
                //     and UnicodeEscape(unit).
                CodePoint::Unicode(c) if c < '\u{0020}' => {
                    product.extend(format!("\\u{:04x}", c as u32).encode_utf16());
                }
                CodePoint::UnpairedSurrogate(surr) => {
                    product.extend(format!("\\u{surr:04x}").encode_utf16());
                }
                // c. Else,
                CodePoint::Unicode(c) => {
                    // i. Set product to the string-concatenation of product and ! UTF16EncodeCodePoint(C).
                    product.extend_from_slice(c.encode_utf16(&mut buf));
                }
            }
        }

        // 3. Set product to the string-concatenation of product and the code unit 0x0022 (QUOTATION MARK).
        product.push('"' as u16);

        // 4. Return product.
        js_string!(&product[..])
    }

    /// `25.5.2.4 SerializeJSONObject ( state, value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-serializejsonobject
    fn serialize_json_object(
        state: &mut StateRecord,
        value: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<JsString> {
        // 1. If state.[[Stack]] contains value, throw a TypeError exception because the structure is cyclical.
        if state.stack.contains(value) {
            return Err(JsNativeError::typ()
                .with_message("cyclic object value")
                .into());
        }

        // 2. Append value to state.[[Stack]].
        state.stack.push(value.clone());

        // 3. Let stepback be state.[[Indent]].
        let stepback = state.indent.clone();

        // 4. Set state.[[Indent]] to the string-concatenation of state.[[Indent]] and state.[[Gap]].
        state.indent = js_string!(&state.indent, &state.gap);

        // 5. If state.[[PropertyList]] is not undefined, then
        let k = if let Some(p) = &state.property_list {
            // a. Let K be state.[[PropertyList]].
            p.clone()
        // 6. Else,
        } else {
            // a. Let K be ? EnumerableOwnPropertyNames(value, key).
            let keys = value.enumerable_own_property_names(PropertyNameKind::Key, context)?;
            // Unwrap is safe, because EnumerableOwnPropertyNames with kind "key" only returns string values.
            keys.iter()
                .map(|v| {
                    v.to_string(context)
                        .expect("EnumerableOwnPropertyNames only returns strings")
                })
                .collect()
        };

        // 7. Let partial be a new empty List.
        let mut partial = Vec::new();

        // 8. For each element P of K, do
        for p in &k {
            // a. Let strP be ? SerializeJSONProperty(state, P, value).
            let str_p = Self::serialize_json_property(state, p.clone(), value, context)?;

            // b. If strP is not undefined, then
            if let Some(str_p) = str_p {
                // i. Let member be QuoteJSONString(P).
                let mut member = Self::quote_json_string(p).to_vec();

                // ii. Set member to the string-concatenation of member and ":".
                member.push(':' as u16);

                // iii. If state.[[Gap]] is not the empty String, then
                if !state.gap.is_empty() {
                    // 1. Set member to the string-concatenation of member and the code unit 0x0020 (SPACE).
                    member.push(' ' as u16);
                }

                // iv. Set member to the string-concatenation of member and strP.
                member.extend_from_slice(&str_p);

                // v. Append member to partial.
                partial.push(member);
            }
        }

        // 9. If partial is empty, then
        let r#final = if partial.is_empty() {
            // a. Let final be "{}".
            js_string!("{}")
        // 10. Else,
        } else {
            // a. If state.[[Gap]] is the empty String, then
            if state.gap.is_empty() {
                // i. Let properties be the String value formed by concatenating all the element Strings of partial
                //    with each adjacent pair of Strings separated with the code unit 0x002C (COMMA).
                //    A comma is not inserted either before the first String or after the last String.
                // ii. Let final be the string-concatenation of "{", properties, and "}".
                let separator = utf16!(",");
                let result = once(utf16!("{"))
                    .chain(Itertools::intersperse(
                        partial.iter().map(Vec::as_slice),
                        separator,
                    ))
                    .chain(once(utf16!("}")))
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                js_string!(&result[..])
            // b. Else,
            } else {
                // i. Let separator be the string-concatenation of the code unit 0x002C (COMMA),
                //    the code unit 0x000A (LINE FEED), and state.[[Indent]].
                let mut separator = utf16!(",\n").to_vec();
                separator.extend_from_slice(&state.indent);
                // ii. Let properties be the String value formed by concatenating all the element Strings of partial
                //     with each adjacent pair of Strings separated with separator.
                //     The separator String is not inserted either before the first String or after the last String.
                // iii. Let final be the string-concatenation of "{", the code
                //      unit 0x000A (LINE FEED), state.[[Indent]], properties,
                //      the code unit 0x000A (LINE FEED), stepback, and "}".
                let result = [utf16!("{\n"), &state.indent[..]]
                    .into_iter()
                    .chain(Itertools::intersperse(
                        partial.iter().map(Vec::as_slice),
                        &separator,
                    ))
                    .chain([utf16!("\n"), &stepback[..], utf16!("}")].into_iter())
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                js_string!(&result[..])
            }
        };

        // 11. Remove the last element of state.[[Stack]].
        state.stack.pop();

        // 12. Set state.[[Indent]] to stepback.
        state.indent = stepback;

        // 13. Return final.
        Ok(r#final)
    }

    /// `25.5.2.5 SerializeJSONArray ( state, value )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-serializejsonarray
    fn serialize_json_array(
        state: &mut StateRecord,
        value: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<JsString> {
        // 1. If state.[[Stack]] contains value, throw a TypeError exception because the structure is cyclical.
        if state.stack.contains(value) {
            return Err(JsNativeError::typ()
                .with_message("cyclic object value")
                .into());
        }

        // 2. Append value to state.[[Stack]].
        state.stack.push(value.clone());

        // 3. Let stepback be state.[[Indent]].
        let stepback = state.indent.clone();

        // 4. Set state.[[Indent]] to the string-concatenation of state.[[Indent]] and state.[[Gap]].
        state.indent = js_string!(&state.indent, &state.gap);

        // 5. Let partial be a new empty List.
        let mut partial = Vec::new();

        // 6. Let len be ? LengthOfArrayLike(value).
        let len = value.length_of_array_like(context)?;

        // 7. Let index be 0.
        let mut index = 0;

        // 8. Repeat, while index < len,
        while index < len {
            // a. Let strP be ? SerializeJSONProperty(state, ! ToString(𝔽(index)), value).
            let str_p =
                Self::serialize_json_property(state, index.to_string().into(), value, context)?;

            // b. If strP is undefined, then
            if let Some(str_p) = str_p {
                // i. Append strP to partial.
                partial.push(Cow::Owned(str_p.to_vec()));
            // c. Else,
            } else {
                // i. Append "null" to partial.
                partial.push(Cow::Borrowed(utf16!("null")));
            }

            // d. Set index to index + 1.
            index += 1;
        }

        // 9. If partial is empty, then
        let r#final = if partial.is_empty() {
            // a. Let final be "[]".
            js_string!("[]")
        // 10. Else,
        } else {
            // a. If state.[[Gap]] is the empty String, then
            if state.gap.is_empty() {
                // i. Let properties be the String value formed by concatenating all the element Strings of partial
                //    with each adjacent pair of Strings separated with the code unit 0x002C (COMMA).
                //    A comma is not inserted either before the first String or after the last String.
                // ii. Let final be the string-concatenation of "[", properties, and "]".
                let separator = utf16!(",");
                let result = once(utf16!("["))
                    .chain(Itertools::intersperse(
                        partial.iter().map(Cow::as_ref),
                        separator,
                    ))
                    .chain(once(utf16!("]")))
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                js_string!(&result[..])
            // b. Else,
            } else {
                // i. Let separator be the string-concatenation of the code unit 0x002C (COMMA),
                //    the code unit 0x000A (LINE FEED), and state.[[Indent]].
                let mut separator = utf16!(",\n").to_vec();
                separator.extend_from_slice(&state.indent);
                // ii. Let properties be the String value formed by concatenating all the element Strings of partial
                //     with each adjacent pair of Strings separated with separator.
                //     The separator String is not inserted either before the first String or after the last String.
                // iii. Let final be the string-concatenation of "[", the code unit 0x000A (LINE FEED), state.[[Indent]], properties, the code unit 0x000A (LINE FEED), stepback, and "]".
                let result = [utf16!("[\n"), &state.indent[..]]
                    .into_iter()
                    .chain(Itertools::intersperse(
                        partial.iter().map(Cow::as_ref),
                        &separator,
                    ))
                    .chain([utf16!("\n"), &stepback[..], utf16!("]")].into_iter())
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                js_string!(&result[..])
            }
        };

        // 11. Remove the last element of state.[[Stack]].
        state.stack.pop();

        // 12. Set state.[[Indent]] to stepback.
        state.indent = stepback;

        // 13. Return final.
        Ok(r#final)
    }
}

struct StateRecord {
    replacer_function: Option<JsObject>,
    stack: Vec<JsObject>,
    indent: JsString,
    gap: JsString,
    property_list: Option<Vec<JsString>>,
}
