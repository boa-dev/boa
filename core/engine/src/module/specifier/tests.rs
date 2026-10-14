use std::path::Path;

use super::*;

#[test]
fn components_next() {
    let specifier = OwnedSpecifier::from_string("foo/bar");

    let mut comps = specifier.components();
    assert_eq!(comps.next(), Some(Component::Root));
    assert_eq!(comps.next(), Some(Component::Normal("foo")));
    assert_eq!(comps.next(), Some(Component::Normal("bar")));
    assert_eq!(comps.next(), None);
    assert_eq!(comps.next_back(), None);

    let specifier = OwnedSpecifier::from_string("./foo/bar");
    let mut comps = specifier.components();
    assert_eq!(comps.next(), Some(Component::Current));
    assert_eq!(comps.next(), Some(Component::Normal("foo")));
    assert_eq!(comps.next(), Some(Component::Normal("bar")));
    assert_eq!(comps.next(), None);
    assert_eq!(comps.next_back(), None);
}

#[test]
fn components_back() {
    let specifier = OwnedSpecifier::from_string("foo/bar");

    let mut comps = specifier.components();
    assert_eq!(comps.next_back(), Some(Component::Normal("bar")));
    assert_eq!(comps.next_back(), Some(Component::Normal("foo")));
    assert_eq!(comps.next_back(), Some(Component::Root));
    assert_eq!(comps.next_back(), None);
    assert_eq!(comps.next(), None);

    let specifier = OwnedSpecifier::from_string("./foo/bar");
    let mut comps = specifier.components();
    assert_eq!(comps.next_back(), Some(Component::Normal("bar")));
    assert_eq!(comps.next_back(), Some(Component::Normal("foo")));
    assert_eq!(comps.next_back(), Some(Component::Current));
    assert_eq!(comps.next_back(), None);
    assert_eq!(comps.next(), None);
}

#[test]
fn components_mixed() {
    let specifier = OwnedSpecifier::from_string("a/b/c/d/e/v/w/x/y/z");

    let mut comps = specifier.components();
    assert_eq!(comps.next(), Some(Component::Root));
    assert_eq!(comps.next_back(), Some(Component::Normal("z")));
    assert_eq!(comps.next(), Some(Component::Normal("a")));
    assert_eq!(comps.next_back(), Some(Component::Normal("y")));
    assert_eq!(comps.next(), Some(Component::Normal("b")));
    assert_eq!(comps.next_back(), Some(Component::Normal("x")));
    assert_eq!(comps.next(), Some(Component::Normal("c")));
    assert_eq!(comps.next_back(), Some(Component::Normal("w")));
    assert_eq!(comps.next(), Some(Component::Normal("d")));
    assert_eq!(comps.next_back(), Some(Component::Normal("v")));
    assert_eq!(comps.next(), Some(Component::Normal("e")));
    assert_eq!(comps.next_back(), Some(Component::Root));
    assert_eq!(comps.next(), None);
    assert_eq!(comps.next_back(), None);
}

#[test]
fn components_empty() {
    let specifier = OwnedSpecifier::from_string("");
    let mut comps = specifier.components();
    assert_eq!(comps.next(), Some(Component::Root));
    assert_eq!(comps.next(), None);
    assert_eq!(comps.next_back(), None);

    let specifier = OwnedSpecifier::from_string("///////");
    let mut comps = specifier.components();
    assert_eq!(comps.next(), Some(Component::Root));
    assert_eq!(comps.next(), None);
    assert_eq!(comps.next_back(), None);

    let specifier = OwnedSpecifier::from_string("");
    let mut comps = specifier.components();
    assert_eq!(comps.next_back(), Some(Component::Root));
    assert_eq!(comps.next_back(), None);
    assert_eq!(comps.next(), None);

    let specifier = OwnedSpecifier::from_string("///////");
    let mut comps = specifier.components();
    assert_eq!(comps.next_back(), Some(Component::Root));
    assert_eq!(comps.next_back(), None);
    assert_eq!(comps.next(), None);
}

#[test]
fn as_specifier() {
    let specifier = OwnedSpecifier::from_string("/foo/bar/baz");
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");
    let specifier = OwnedSpecifier::from_string("foo/bar/baz");
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");
    let specifier = OwnedSpecifier::from_string("//foo/bar/baz");
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");
    let specifier = OwnedSpecifier::from_string("./foo/bar/baz");
    assert_eq!(specifier.as_specifier(), "./foo/bar/baz");
    let specifier = OwnedSpecifier::from_string(".//foo/bar/baz");
    assert_eq!(specifier.as_specifier(), "./foo/bar/baz");
}

#[test]
fn from_path() {
    let specifier = OwnedSpecifier::try_from_path(Path::new("/foo/bar/baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");
    let specifier = OwnedSpecifier::try_from_path(Path::new("./foo/bar/baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "./foo/bar/baz");
    let specifier = OwnedSpecifier::try_from_path(Path::new("///foo/bar/baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");
    let specifier = OwnedSpecifier::try_from_path(Path::new("foo/bar/baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "foo/bar/baz");
}

#[test]
fn parent() {
    let specifier = OwnedSpecifier::from_string("/foo/bar/baz");
    assert_eq!(specifier.parent().unwrap(), "/foo/bar");
    assert_eq!(specifier.parent().unwrap().parent().unwrap(), "/foo");

    let specifier = OwnedSpecifier::from_string("./bar/baz");
    assert_eq!(specifier.parent().unwrap(), "./bar");
    assert_eq!(specifier.parent().unwrap().parent().unwrap(), ".");
}

#[test]
fn normalize() {
    let specifier = OwnedSpecifier::from_string("////a/b/./c/./../././c//////d/././//e///");
    assert_eq!(specifier.normalize().as_specifier(), "/a/b/c/d/e");

    let specifier = OwnedSpecifier::from_string("./../a/b/c/d/e");
    assert_eq!(specifier.normalize().as_specifier(), "../a/b/c/d/e");

    let specifier = OwnedSpecifier::from_string("./../a/../../b/c/d/e");
    assert_eq!(specifier.normalize().as_specifier(), "../../b/c/d/e");

    let specifier = OwnedSpecifier::from_string(".././a/b/c/d/e");
    assert_eq!(specifier.normalize().as_specifier(), "../a/b/c/d/e");
}

#[cfg(target_family = "unix")]
#[test]
fn path_unix() {
    let specifier = OwnedSpecifier::from_string("/foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("/foo/bar/baz"));
    let specifier = OwnedSpecifier::from_string("./foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("./foo/bar/baz"));
    let specifier = OwnedSpecifier::from_string("///foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("/foo/bar/baz"));
    let specifier = OwnedSpecifier::from_string("foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("/foo/bar/baz"));

    let specifier = OwnedSpecifier::try_from_path(Path::new("/foo/bar/baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");

    let specifier = OwnedSpecifier::try_from_path(Path::new("./foo/../bar/baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "./foo/../bar/baz");
}

#[cfg(target_family = "windows")]
#[test]
fn path_windows() {
    let specifier = OwnedSpecifier::from_string("/foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("\\foo\\bar\\baz"));
    let specifier = OwnedSpecifier::from_string("./foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new(".\\foo\\bar\\baz"));
    let specifier = OwnedSpecifier::from_string("///foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("\\foo\\bar\\baz"));
    let specifier = OwnedSpecifier::from_string("foo/bar/baz");
    assert_eq!(specifier.to_path_buf(), Path::new("\\foo\\bar\\baz"));

    let specifier = OwnedSpecifier::try_from_path(Path::new("\\foo\\bar\\baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "/foo/bar/baz");

    let specifier = OwnedSpecifier::try_from_path(Path::new(".\\foo\\..\\bar\\baz")).unwrap();
    assert_eq!(specifier.as_specifier(), "./foo/../bar/baz");
}
