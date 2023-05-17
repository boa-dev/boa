use std::fmt::Write;

use boa_profiler::Profiler;
use icu_list::{provider::AndListV1Marker, ListFormatter, ListLength};
use icu_locid::Locale;
use icu_provider::DataLocale;

use crate::{
    builtins::{Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::Attribute,
    realm::Realm,
    string::utf16,
    symbol::JsSymbol,
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
};

use super::{
    locale::{canonicalize_locale_list, resolve_locale, supported_locales},
    options::{get_option, get_options_object, IntlOptions, LocaleMatcher},
    Service,
};

mod options;
pub(crate) use options::*;
pub struct ListFormat {
    locale: Locale,
    typ: ListFormatType,
    style: ListLength,
    formatter: ListFormatter,
}

impl std::fmt::Debug for ListFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListFormat")
            .field("locale", &self.locale)
            .field("typ", &self.typ)
            .field("style", &self.style)
            .field("formatter", &"ListFormatter")
            .finish()
    }
}

impl Service for ListFormat {
    type LangMarker = AndListV1Marker;

    type LocaleOptions = ();
}

impl IntrinsicObject for ListFormat {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::supported_locales_of, "supportedLocalesOf", 1)
            .property(
                JsSymbol::to_string_tag(),
                "Intl.ListFormat",
                Attribute::CONFIGURABLE,
            )
            .method(Self::format, "format", 1)
            .method(Self::format_to_parts, "formatToParts", 1)
            .method(Self::resolved_options, "resolvedOptions", 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ListFormat {
    const NAME: &'static str = "ListFormat";
}

impl BuiltInConstructor for ListFormat {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::list_format;

    /// Constructor [`Intl.ListFormat ( [ locales [ , options ] ] )`][spec].
    ///
    /// Constructor for `ListFormat` objects.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.ListFormat` constructor without `new`")
                .into());
        }

        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 3. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(options)?;

        // 5. Let opt be a new Record.
        // 6. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
        let matcher =
            get_option::<LocaleMatcher>(&options, utf16!("localeMatcher"), false, context)?
                .unwrap_or_default();

        // 7. Set opt.[[localeMatcher]] to matcher.
        // 8. Let localeData be %ListFormat%.[[LocaleData]].
        // 9. Let r be ResolveLocale(%ListFormat%.[[AvailableLocales]], requestedLocales, opt, %ListFormat%.[[RelevantExtensionKeys]], localeData).
        // 10. Set listFormat.[[Locale]] to r.[[locale]].
        let locale = resolve_locale::<Self>(
            &requested_locales,
            &mut IntlOptions {
                matcher,
                ..Default::default()
            },
            context.icu(),
        );

        // 11. Let type be ? GetOption(options, "type", string, « "conjunction", "disjunction", "unit" », "conjunction").
        // 12. Set listFormat.[[Type]] to type.
        let typ = get_option::<ListFormatType>(&options, utf16!("type"), false, context)?
            .unwrap_or_default();

        // 13. Let style be ? GetOption(options, "style", string, « "long", "short", "narrow" », "long").
        // 14. Set listFormat.[[Style]] to style.
        let style = get_option::<ListLength>(&options, utf16!("style"), false, context)?
            .unwrap_or(ListLength::Wide);

        // 15. Let dataLocale be r.[[dataLocale]].
        // 16. Let dataLocaleData be localeData.[[<dataLocale>]].
        // 17. Let dataLocaleTypes be dataLocaleData.[[<type>]].
        // 18. Set listFormat.[[Templates]] to dataLocaleTypes.[[<style>]].

        // 2. Let listFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%ListFormat.prototype%", « [[InitializedListFormat]], [[Locale]], [[Type]], [[Style]], [[Templates]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::list_format, context)?;
        let list_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::list_format(Self {
                formatter: context
                    .icu()
                    .provider()
                    .try_new_list_formatter(&DataLocale::from(&locale), typ, style)
                    .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?,
                locale,
                typ,
                style,
            }),
        );

        // 19. Return listFormat.
        Ok(list_format.into())
    }
}

impl ListFormat {
    /// [`Intl.ListFormat.supportedLocalesOf ( locales [ , options ] )`][spec].
    ///
    /// Returns an array containing those of the provided locales that are supported in list
    /// formatting without having to fall back to the runtime's default locale.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat.supportedLocalesOf
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/supportedLocalesOf
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. Let availableLocales be %ListFormat%.[[AvailableLocales]].
        // 2. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // 3. Return ? SupportedLocales(availableLocales, requestedLocales, options).
        supported_locales::<<Self as Service>::LangMarker>(&requested_locales, options, context)
            .map(JsValue::from)
    }

    /// [`Intl.ListFormat.prototype.format ( list )`][spec].
    ///
    /// Returns a language-specific formatted string representing the elements of the list.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat.prototype.format
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/format
    fn format(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let lf be the this value.
        // 2. Perform ? RequireInternalSlot(lf, [[InitializedListFormat]]).
        let lf = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`format` can only be called on a `ListFormat` object")
        })?;
        let lf = lf.as_list_format().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`format` can only be called on a `ListFormat` object")
        })?;

        // 3. Let stringList be ? StringListFromIterable(list).
        // TODO: support for UTF-16 unpaired surrogates formatting
        let strings = string_list_from_iterable(args.get_or_undefined(0), context)?;

        // 4. Return ! FormatList(lf, stringList).
        Ok(lf
            .formatter
            .format_to_string(strings.into_iter().map(|s| s.to_std_string_escaped()))
            .into())
    }

    /// [`Intl.ListFormat.prototype.formatToParts ( list )`][spec].
    ///
    /// Returns a language-specific formatted string representing the elements of the list.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat.prototype.formatToParts
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/formatToParts
    fn format_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // TODO: maybe try to move this into icu4x?
        use writeable::{PartsWrite, Writeable};

        #[derive(Debug, Clone)]
        enum Part {
            Literal(String),
            Element(String),
        }

        impl Part {
            const fn typ(&self) -> &'static str {
                match self {
                    Self::Literal(_) => "literal",
                    Self::Element(_) => "element",
                }
            }

            #[allow(clippy::missing_const_for_fn)]
            fn value(self) -> String {
                match self {
                    Self::Literal(s) | Self::Element(s) => s,
                }
            }
        }

        #[derive(Debug, Clone)]
        struct WriteString(String);

        impl Write for WriteString {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                self.0.write_str(s)
            }

            fn write_char(&mut self, c: char) -> std::fmt::Result {
                self.0.write_char(c)
            }
        }

        impl PartsWrite for WriteString {
            type SubPartsWrite = Self;

            fn with_part(
                &mut self,
                _part: writeable::Part,
                mut f: impl FnMut(&mut Self::SubPartsWrite) -> std::fmt::Result,
            ) -> std::fmt::Result {
                f(self)
            }
        }

        #[derive(Debug, Clone)]
        struct PartsCollector(Vec<Part>);

        impl Write for PartsCollector {
            fn write_str(&mut self, _: &str) -> std::fmt::Result {
                Ok(())
            }
        }

        impl PartsWrite for PartsCollector {
            type SubPartsWrite = WriteString;

            fn with_part(
                &mut self,
                part: writeable::Part,
                mut f: impl FnMut(&mut Self::SubPartsWrite) -> core::fmt::Result,
            ) -> core::fmt::Result {
                assert!(part.category == "list");
                let mut string = WriteString(String::new());
                f(&mut string)?;
                if !string.0.is_empty() {
                    match part.value {
                        "element" => self.0.push(Part::Element(string.0)),
                        "literal" => self.0.push(Part::Literal(string.0)),
                        _ => unreachable!(),
                    };
                }
                Ok(())
            }
        }

        // 1. Let lf be the this value.
        // 2. Perform ? RequireInternalSlot(lf, [[InitializedListFormat]]).
        let lf = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`formatToParts` can only be called on a `ListFormat` object")
        })?;
        let lf = lf.as_list_format().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`formatToParts` can only be called on a `ListFormat` object")
        })?;

        // 3. Let stringList be ? StringListFromIterable(list).
        // TODO: support for UTF-16 unpaired surrogates formatting
        let strings = string_list_from_iterable(args.get_or_undefined(0), context)?
            .into_iter()
            .map(|s| s.to_std_string_escaped());

        // 4. Return ! FormatListToParts(lf, stringList).

        // Abstract operation `FormatListToParts ( listFormat, list )`
        // https://tc39.es/ecma402/#sec-formatlisttoparts

        // 1. Let parts be ! CreatePartsFromList(listFormat, list).
        let mut parts = PartsCollector(Vec::new());
        lf.formatter
            .format(strings)
            .write_to_parts(&mut parts)
            .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;

        // 2. Let result be ! ArrayCreate(0).
        let result = Array::array_create(0, None, context)
            .expect("creating an empty array with default proto must not fail");

        // 3. Let n be 0.
        // 4. For each Record { [[Type]], [[Value]] } part in parts, do
        for (n, part) in parts.0.into_iter().enumerate() {
            // a. Let O be OrdinaryObjectCreate(%Object.prototype%).
            let o = context
                .intrinsics()
                .templates()
                .ordinary_object()
                .create(ObjectData::ordinary(), vec![]);

            // b. Perform ! CreateDataPropertyOrThrow(O, "type", part.[[Type]]).
            o.create_data_property_or_throw(utf16!("type"), part.typ(), context)
                .expect("operation must not fail per the spec");

            // c. Perform ! CreateDataPropertyOrThrow(O, "value", part.[[Value]]).
            o.create_data_property_or_throw(utf16!("value"), part.value(), context)
                .expect("operation must not fail per the spec");

            // d. Perform ! CreateDataPropertyOrThrow(result, ! ToString(n), O).
            result
                .create_data_property_or_throw(n, o, context)
                .expect("operation must not fail per the spec");

            // e. Increment n by 1.
        }

        // 5. Return result.
        Ok(result.into())
    }

    /// [`Intl.ListFormat.prototype.resolvedOptions ( )`][spec].
    ///
    /// Returns a new object with properties reflecting the locale and style formatting options
    /// computed during the construction of the current `Intl.ListFormat` object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat.prototype.resolvedoptions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/resolvedOptions
    fn resolved_options(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let lf be the this value.
        // 2. Perform ? RequireInternalSlot(lf, [[InitializedListFormat]]).
        let lf = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`resolvedOptions` can only be called on a `ListFormat` object")
        })?;
        let lf = lf.as_list_format().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("`resolvedOptions` can only be called on a `ListFormat` object")
        })?;

        // 3. Let options be OrdinaryObjectCreate(%Object.prototype%).
        let options = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(ObjectData::ordinary(), vec![]);

        // 4. For each row of Table 11, except the header row, in table order, do
        //     a. Let p be the Property value of the current row.
        //     b. Let v be the value of lf's internal slot whose name is the Internal Slot value of the current row.
        //     c. Assert: v is not undefined.
        //     d. Perform ! CreateDataPropertyOrThrow(options, p, v).
        options
            .create_data_property_or_throw(utf16!("locale"), lf.locale.to_string(), context)
            .expect("operation must not fail per the spec");
        options
            .create_data_property_or_throw(
                utf16!("type"),
                match lf.typ {
                    ListFormatType::Conjunction => "conjunction",
                    ListFormatType::Disjunction => "disjunction",
                    ListFormatType::Unit => "unit",
                },
                context,
            )
            .expect("operation must not fail per the spec");
        options
            .create_data_property_or_throw(
                utf16!("style"),
                match lf.style {
                    ListLength::Wide => "long",
                    ListLength::Short => "short",
                    ListLength::Narrow => "narrow",
                    _ => unreachable!(),
                },
                context,
            )
            .expect("operation must not fail per the spec");

        // 5. Return options.
        Ok(options.into())
    }
}

/// Abstract operation [`StringListFromIterable ( iterable )`][spec]
///
/// [spec]: https://tc39.es/ecma402/#sec-createstringlistfromiterable
fn string_list_from_iterable(
    iterable: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<Vec<JsString>> {
    // 1. If iterable is undefined, then
    if iterable.is_undefined() {
        //     a. Return a new empty List.
        return Ok(Vec::new());
    }

    // 2. Let iteratorRecord be ? GetIterator(iterable).
    let mut iterator = iterable.get_iterator(context, None, None)?;

    // 3. Let list be a new empty List.
    let mut list = Vec::new();

    // 4. Let next be true.
    // 5. Repeat, while next is not false,
    //     a. Set next to ? IteratorStep(iteratorRecord).
    //     b. If next is not false, then
    while !iterator.step(context)? {
        let item = iterator.value(context)?;
        //    i. Let nextValue be ? IteratorValue(next).
        //    ii. If Type(nextValue) is not String, then
        let Some(s) = item.as_string().cloned() else {
            //    1. Let error be ThrowCompletion(a newly created TypeError object).
            //    2. Return ? IteratorClose(iteratorRecord, error).
            return Err(iterator
                .close(
                    Err(JsNativeError::typ()
                    .with_message("StringListFromIterable: can only format strings into a list")
                    .into()),
                    context,
                )
                .expect_err("Should return the provided error"));
        };

        //    iii. Append nextValue to the end of the List list.
        list.push(s);
    }

    // 6. Return list.
    Ok(list)
}
