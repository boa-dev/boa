use super::Attribute;

#[test]
fn writable() {
    let attribute = Attribute::WRITABLE;

    assert!(attribute.has_writable());
    assert!(attribute.writable());
}

#[test]
fn enumerable() {
    let attribute = Attribute::ENUMERABLE;

    assert!(attribute.has_enumerable());
    assert!(attribute.enumerable());
}

#[test]
fn configurable() {
    let attribute = Attribute::CONFIGURABLE;

    assert!(attribute.has_configurable());
    assert!(attribute.configurable());
}

#[test]
fn writable_and_enumerable() {
    let attribute = Attribute::WRITABLE | Attribute::ENUMERABLE;

    assert!(attribute.has_writable());
    assert!(attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(attribute.enumerable());

    assert!(!attribute.has_configurable());
}

#[test]
fn enumerable_configurable() {
    let attribute = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;

    assert!(!attribute.has_writable());

    assert!(attribute.has_enumerable());
    assert!(attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(attribute.configurable());
}

#[test]
fn writable_enumerable_configurable() {
    let attribute = Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE;

    assert!(attribute.has_writable());
    assert!(attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(attribute.configurable());
}

#[test]
fn default() {
    let attribute = Attribute::default();

    assert!(attribute.has_writable());
    assert!(attribute.has_enumerable());
    assert!(attribute.has_configurable());
}

#[test]
fn clear() {
    let mut attribute = Attribute::default();

    attribute.clear();

    assert!(!attribute.has_writable());
    assert!(!attribute.has_enumerable());
    assert!(!attribute.has_configurable());

    assert!(attribute.is_empty());
}

#[test]
fn set_writable_to_true() {
    let mut attribute = Attribute::default();

    attribute.set_writable(true);

    assert!(attribute.has_writable());
    assert!(attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(!attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(!attribute.configurable());
}

#[test]
fn set_writable_to_false() {
    let mut attribute = Attribute::default();

    attribute.set_writable(false);

    assert!(attribute.has_writable());
    assert!(!attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(!attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(!attribute.configurable());
}

#[test]
fn set_enumerable_to_true() {
    let mut attribute = Attribute::default();

    attribute.set_enumerable(true);

    assert!(attribute.has_writable());
    assert!(!attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(!attribute.configurable());
}

#[test]
fn set_enumerable_to_false() {
    let mut attribute = Attribute::default();

    attribute.set_enumerable(false);

    assert!(attribute.has_writable());
    assert!(!attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(!attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(!attribute.configurable());
}

#[test]
fn set_configurable_to_true() {
    let mut attribute = Attribute::default();

    attribute.set_configurable(true);

    assert!(attribute.has_writable());
    assert!(!attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(!attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(attribute.configurable());
}

#[test]
fn set_configurable_to_false() {
    let mut attribute = Attribute::default();

    attribute.set_configurable(false);

    assert!(attribute.has_writable());
    assert!(!attribute.writable());
    assert!(attribute.has_enumerable());
    assert!(!attribute.enumerable());
    assert!(attribute.has_configurable());
    assert!(!attribute.configurable());
}
