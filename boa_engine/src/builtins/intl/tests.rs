use crate::{builtins::Array, Context, JsString, JsValue};

#[test]
fn evaluate_extensions() {
    let locale_no_ext = "ja-Jpan-JP".to_string();
    let ext = "-u-ca-japanese-hc-h12".to_string();
    let locale_str = locale_no_ext.clone() + &ext;
    assert_eq!(
        crate::builtins::intl::get_leftmost_ext_pos(&locale_str),
        locale_no_ext.len()
    );
    assert_eq!(
        crate::builtins::intl::trim_extensions(&locale_str),
        locale_no_ext
    );
    assert_eq!(crate::builtins::intl::extract_extensions(&locale_str), ext);

    let locale_no_ext = "ko-Kore-KR".to_string();
    let ext = "".to_string();
    let locale_str = locale_no_ext.clone() + &ext;
    assert_eq!(
        crate::builtins::intl::get_leftmost_ext_pos(&locale_str),
        locale_no_ext.len()
    );
    assert_eq!(
        crate::builtins::intl::trim_extensions(&locale_str),
        locale_no_ext
    );
    assert_eq!(crate::builtins::intl::extract_extensions(&locale_str), ext);

    let locale_no_ext = "de-DE".to_string();
    let ext = "-u-co-phonebk-ka-shifted".to_string();
    let locale_str = locale_no_ext.clone() + &ext;
    assert_eq!(
        crate::builtins::intl::get_leftmost_ext_pos(&locale_str),
        locale_no_ext.len()
    );
    assert_eq!(
        crate::builtins::intl::trim_extensions(&locale_str),
        locale_no_ext
    );
    assert_eq!(crate::builtins::intl::extract_extensions(&locale_str), ext);

    let locale_no_ext = "ar".to_string();
    let ext = "-u-nu-native".to_string();
    let locale_str = locale_no_ext.clone() + &ext;
    assert_eq!(
        crate::builtins::intl::get_leftmost_ext_pos(&locale_str),
        locale_no_ext.len()
    );
    assert_eq!(
        crate::builtins::intl::trim_extensions(&locale_str),
        locale_no_ext
    );
    assert_eq!(crate::builtins::intl::extract_extensions(&locale_str), ext);

    let locale_no_ext = "und".to_string();
    let ext = "-u-cu-usd-va-posix".to_string();
    let locale_str = locale_no_ext.clone() + &ext;
    assert_eq!(
        crate::builtins::intl::get_leftmost_ext_pos(&locale_str),
        locale_no_ext.len()
    );
    assert_eq!(
        crate::builtins::intl::trim_extensions(&locale_str),
        locale_no_ext
    );
    assert_eq!(crate::builtins::intl::extract_extensions(&locale_str), ext);
}

#[test]
fn best_avail_loc() {
    let mut context = Context::default();

    let no_extensions_locale = "en-US".to_string();
    let locales_list = vec![];
    let available_locales =
        JsValue::Object(Array::create_array_from_list(locales_list, &mut context));
    assert_eq!(
        crate::builtins::intl::best_available_locale(
            &available_locales,
            &no_extensions_locale,
            &mut context
        ),
        "undefined".to_string()
    );

    let no_extensions_locale = "de-DE".to_string();
    let locales_list = vec![JsString::new(no_extensions_locale.clone())];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        locales_list.into_iter().map(Into::into),
        &mut context,
    ));
    assert_eq!(
        crate::builtins::intl::best_available_locale(
            &available_locales,
            &no_extensions_locale,
            &mut context
        ),
        no_extensions_locale.clone()
    );

    let locale_part = "fr".to_string();
    let no_extensions_locale = locale_part.clone() + &"-CA".to_string();
    let locales_list = vec![JsString::new(locale_part.clone())];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        locales_list.into_iter().map(Into::into),
        &mut context,
    ));
    assert_eq!(
        crate::builtins::intl::best_available_locale(
            &available_locales,
            &no_extensions_locale,
            &mut context
        ),
        locale_part.clone()
    );

    let ja_kana_t = "ja-Kana-JP-t".to_string();
    let ja_kana = "ja-Kana-JP".to_string();
    let no_extensions_locale = "ja-Kana-JP-t-it-latn-it".to_string();
    let locales_list = vec![
        JsString::new(ja_kana_t.clone()),
        JsString::new(ja_kana.clone()),
    ];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        locales_list.into_iter().map(Into::into),
        &mut context,
    ));
    assert_eq!(
        crate::builtins::intl::best_available_locale(
            &available_locales,
            &no_extensions_locale,
            &mut context
        ),
        ja_kana.clone()
    );
}

#[test]
fn lookup_match() {
    let mut context = Context::default();

    // available: [], requested: []
    let avail_locales_list = vec![];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        avail_locales_list,
        &mut context,
    ));

    let requested_locales_list = vec![];
    let requested_locales = JsValue::Object(Array::create_array_from_list(
        requested_locales_list,
        &mut context,
    ));

    let matcher =
        crate::builtins::intl::lookup_matcher(&available_locales, &requested_locales, &mut context);
    assert_eq!(matcher.locale, crate::builtins::intl::default_locale());
    assert_eq!(matcher.extension, "");

    // available: [de-DE], requested: []
    let avail_locales_list = vec![JsValue::String(JsString::new("de-DE"))];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        avail_locales_list,
        &mut context,
    ));

    let requested_locales_list = vec![];
    let requested_locales = JsValue::Object(Array::create_array_from_list(
        requested_locales_list,
        &mut context,
    ));

    let matcher =
        crate::builtins::intl::lookup_matcher(&available_locales, &requested_locales, &mut context);
    assert_eq!(matcher.locale, crate::builtins::intl::default_locale());
    assert_eq!(matcher.extension, "");

    // available: [fr-FR], requested: [fr-FR-u-hc-h12]
    let avail_locales_list = vec![JsValue::String(JsString::new("fr-FR"))];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        avail_locales_list,
        &mut context,
    ));

    let requested_locales_list = vec![JsValue::String(JsString::new("fr-FR-u-hc-h12"))];
    let requested_locales = JsValue::Object(Array::create_array_from_list(
        requested_locales_list,
        &mut context,
    ));

    let matcher =
        crate::builtins::intl::lookup_matcher(&available_locales, &requested_locales, &mut context);
    assert_eq!(matcher.locale, "fr-FR");
    assert_eq!(matcher.extension, "");

    // available: [es-ES], requested: [es-ES]
    let avail_locales_list = vec![JsValue::String(JsString::new("es-ES"))];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        avail_locales_list,
        &mut context,
    ));

    let requested_locales_list = vec![JsValue::String(JsString::new("es-ES"))];
    let requested_locales = JsValue::Object(Array::create_array_from_list(
        requested_locales_list,
        &mut context,
    ));

    let matcher = crate::builtins::intl::best_fit_matcher(
        &available_locales,
        &requested_locales,
        &mut context,
    );
    assert_eq!(matcher.locale, "es-ES");
    assert_eq!(matcher.extension, "");
}

#[test]
fn insert_unicode_ext() {
    let locale = "hu-HU".to_string();
    let ext = "".to_string();
    assert_eq!(
        crate::builtins::intl::insert_unicode_extension_and_canonicalize(&locale, &ext),
        locale
    );
}

#[test]
fn find_elem_in_js_arr() {
    let mut context = Context::default();

    let locales_list = vec![
        JsString::new("es"),
        JsString::new("fr"),
        JsString::new("de"),
    ];
    let available_locales = JsValue::Object(Array::create_array_from_list(
        locales_list.into_iter().map(Into::into),
        &mut context,
    ));

    assert_eq!(
        crate::builtins::intl::js_arr_contains_str(
            &available_locales,
            &"es".to_string(),
            &mut context
        ),
        true
    );
    assert_eq!(
        crate::builtins::intl::js_arr_contains_str(
            &available_locales,
            &"en".to_string(),
            &mut context
        ),
        false
    );
}

#[test]
fn uni_ext_comp() {
    let ext = "-u-ca-japanese-hc-h12".to_string();
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes.is_empty(), true);
    assert_eq!(components.keywords.len(), 2);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "japanese");
    assert_eq!(components.keywords[1].key, "hc");
    assert_eq!(components.keywords[1].value, "h12");

    let ext = "-u-alias-co-phonebk-ka-shifted".to_string();
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes, vec!["alias".to_string()]);
    assert_eq!(components.keywords.len(), 2);
    assert_eq!(components.keywords[0].key, "co");
    assert_eq!(components.keywords[0].value, "phonebk");
    assert_eq!(components.keywords[1].key, "ka");
    assert_eq!(components.keywords[1].value, "shifted");

    let ext = "-u-ca-buddhist-kk-nu-thai".to_string();
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes.is_empty(), true);
    assert_eq!(components.keywords.len(), 3);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "buddhist");
    assert_eq!(components.keywords[1].key, "kk");
    assert_eq!(components.keywords[1].value, "");
    assert_eq!(components.keywords[2].key, "nu");
    assert_eq!(components.keywords[2].value, "thai");

    let ext = "-u-ca-islamic-civil".to_string();
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes.is_empty(), true);
    assert_eq!(components.keywords.len(), 1);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "islamic-civil");
}
