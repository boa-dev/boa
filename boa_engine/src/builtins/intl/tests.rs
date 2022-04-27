use crate::{Context, JsString};

use rustc_hash::FxHashMap;

#[test]
fn best_avail_loc() {
    let no_extensions_locale = JsString::new("en-US");
    let available_locales = Vec::<JsString>::new();
    assert_eq!(
        crate::builtins::intl::best_available_locale(&available_locales, &no_extensions_locale,),
        None
    );

    let no_extensions_locale = JsString::new("de-DE");
    let available_locales = vec![no_extensions_locale.clone()];
    assert_eq!(
        crate::builtins::intl::best_available_locale(&available_locales, &no_extensions_locale,),
        Some(no_extensions_locale)
    );

    let locale_part = "fr".to_string();
    let no_extensions_locale = JsString::new(locale_part.clone() + &"-CA".to_string());
    let available_locales = vec![JsString::new(locale_part.clone())];
    assert_eq!(
        crate::builtins::intl::best_available_locale(&available_locales, &no_extensions_locale,),
        Some(JsString::new(locale_part))
    );

    let ja_kana_t = JsString::new("ja-Kana-JP-t");
    let ja_kana = JsString::new("ja-Kana-JP");
    let no_extensions_locale = JsString::new("ja-Kana-JP-t-it-latn-it");
    let available_locales = vec![ja_kana_t.clone(), ja_kana.clone()];
    assert_eq!(
        crate::builtins::intl::best_available_locale(&available_locales, &no_extensions_locale,),
        Some(ja_kana)
    );
}

#[test]
fn lookup_match() {
    // available: [], requested: []
    let available_locales = Vec::<JsString>::new();
    let requested_locales = Vec::<JsString>::new();

    let matcher = crate::builtins::intl::lookup_matcher(&available_locales, &requested_locales);
    assert_eq!(matcher.locale, crate::builtins::intl::default_locale());
    assert_eq!(matcher.extension, "");

    // available: [de-DE], requested: []
    let available_locales = vec![JsString::new("de-DE")];
    let requested_locales = Vec::<JsString>::new();

    let matcher = crate::builtins::intl::lookup_matcher(&available_locales, &requested_locales);
    assert_eq!(matcher.locale, crate::builtins::intl::default_locale());
    assert_eq!(matcher.extension, "");

    // available: [fr-FR], requested: [fr-FR-u-hc-h12]
    let available_locales = vec![JsString::new("fr-FR")];
    let requested_locales = vec![JsString::new("fr-FR-u-hc-h12")];

    let matcher = crate::builtins::intl::lookup_matcher(&available_locales, &requested_locales);
    assert_eq!(matcher.locale, "fr-FR");
    assert_eq!(matcher.extension, "-u-hc-h12");

    // available: [es-ES], requested: [es-ES]
    let available_locales = vec![JsString::new("es-ES")];
    let requested_locales = vec![JsString::new("es-ES")];

    let matcher = crate::builtins::intl::best_fit_matcher(&available_locales, &requested_locales);
    assert_eq!(matcher.locale, "es-ES");
    assert_eq!(matcher.extension, "");
}

#[test]
fn insert_unicode_ext() {
    let locale = JsString::new("hu-HU");
    let ext = JsString::empty();
    assert_eq!(
        crate::builtins::intl::insert_unicode_extension_and_canonicalize(&locale, &ext),
        locale
    );

    let locale = JsString::new("hu-HU");
    let ext = JsString::new("-u-hc-h12");
    assert_eq!(
        crate::builtins::intl::insert_unicode_extension_and_canonicalize(&locale, &ext),
        JsString::new("hu-HU-u-hc-h12")
    );

    let locale = JsString::new("hu-HU-x-PRIVATE");
    let ext = JsString::new("-u-hc-h12");
    assert_eq!(
        crate::builtins::intl::insert_unicode_extension_and_canonicalize(&locale, &ext),
        JsString::new("hu-HU-u-hc-h12-x-PRIVATE")
    );
}

#[test]
fn uni_ext_comp() {
    let ext = JsString::new("-u-ca-japanese-hc-h12");
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes.is_empty(), true);
    assert_eq!(components.keywords.len(), 2);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "japanese");
    assert_eq!(components.keywords[1].key, "hc");
    assert_eq!(components.keywords[1].value, "h12");

    let ext = JsString::new("-u-alias-co-phonebk-ka-shifted");
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes, vec![JsString::new("alias")]);
    assert_eq!(components.keywords.len(), 2);
    assert_eq!(components.keywords[0].key, "co");
    assert_eq!(components.keywords[0].value, "phonebk");
    assert_eq!(components.keywords[1].key, "ka");
    assert_eq!(components.keywords[1].value, "shifted");

    let ext = JsString::new("-u-ca-buddhist-kk-nu-thai");
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes.is_empty(), true);
    assert_eq!(components.keywords.len(), 3);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "buddhist");
    assert_eq!(components.keywords[1].key, "kk");
    assert_eq!(components.keywords[1].value, "");
    assert_eq!(components.keywords[2].key, "nu");
    assert_eq!(components.keywords[2].value, "thai");

    let ext = JsString::new("-u-ca-islamic-civil");
    let components = crate::builtins::intl::unicode_extension_components(&ext);
    assert_eq!(components.attributes.is_empty(), true);
    assert_eq!(components.keywords.len(), 1);
    assert_eq!(components.keywords[0].key, "ca");
    assert_eq!(components.keywords[0].value, "islamic-civil");
}

#[test]
fn locale_resolution() {
    let mut context = Context::default();

    // test lookup
    let available_locales = Vec::<JsString>::new();
    let requested_locales = Vec::<JsString>::new();
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = crate::builtins::intl::DateTimeFormatRecord {
        locale_matcher: JsString::new("lookup"),
        properties: FxHashMap::default(),
    };

    let locale_record = crate::builtins::intl::resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(
        locale_record.locale,
        crate::builtins::intl::default_locale()
    );
    assert_eq!(
        locale_record.data_locale,
        crate::builtins::intl::default_locale()
    );
    assert_eq!(locale_record.properties.is_empty(), true);

    // test best fit
    let available_locales = Vec::<JsString>::new();
    let requested_locales = Vec::<JsString>::new();
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = crate::builtins::intl::DateTimeFormatRecord {
        locale_matcher: JsString::new("best-fit"),
        properties: FxHashMap::default(),
    };

    let locale_record = crate::builtins::intl::resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(
        locale_record.locale,
        crate::builtins::intl::default_locale()
    );
    assert_eq!(
        locale_record.data_locale,
        crate::builtins::intl::default_locale()
    );
    assert_eq!(locale_record.properties.is_empty(), true);

    // available: [es-ES], requested: [es-ES]
    let available_locales = vec![JsString::new("es-ES")];
    let requested_locales = vec![JsString::new("es-ES")];
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = crate::builtins::intl::DateTimeFormatRecord {
        locale_matcher: JsString::new("lookup"),
        properties: FxHashMap::default(),
    };

    let locale_record = crate::builtins::intl::resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(locale_record.locale, "es-ES");
    assert_eq!(locale_record.data_locale, "es-ES");
    assert_eq!(locale_record.properties.is_empty(), true);

    // available: [zh-CN], requested: []
    let available_locales = vec![JsString::new("zh-CN")];
    let requested_locales = Vec::<JsString>::new();
    let relevant_extension_keys = Vec::<JsString>::new();
    let locale_data = FxHashMap::default();
    let options = crate::builtins::intl::DateTimeFormatRecord {
        locale_matcher: JsString::new("lookup"),
        properties: FxHashMap::default(),
    };

    let locale_record = crate::builtins::intl::resolve_locale(
        &available_locales,
        &requested_locales,
        &options,
        &relevant_extension_keys,
        &locale_data,
        &mut context,
    );
    assert_eq!(
        locale_record.locale,
        crate::builtins::intl::default_locale()
    );
    assert_eq!(
        locale_record.data_locale,
        crate::builtins::intl::default_locale()
    );
    assert_eq!(locale_record.properties.is_empty(), true);
}
