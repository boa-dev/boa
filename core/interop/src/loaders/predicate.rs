//! A [`ModuleLoader`] that uses a predicate to modify the module specifier to load.

use std::fmt::Debug;
use std::path::Path;

use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::{Context, JsResult, JsString, Module};

/// A [`ModuleLoader`] that uses a predicate to modify the module specifier to load,
/// before passing it to the inner module loader.
pub struct PredicateModuleLoader<P, Inner>
where
    P: Fn(Option<&Path>, JsString) -> JsResult<JsString>,
{
    predicate: P,
    inner: Inner,
}

impl<P, Inner> Debug for PredicateModuleLoader<P, Inner>
where
    P: Fn(Option<&Path>, JsString) -> JsResult<JsString>,
    Inner: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PredicateModuleLoader")
            .field("predicate", &"Fn()")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<P, Inner> PredicateModuleLoader<P, Inner>
where
    P: Fn(Option<&Path>, JsString) -> JsResult<JsString>,
{
    pub fn new(predicate: P, inner: Inner) -> Self {
        Self { predicate, inner }
    }
}

impl<P, Inner> ModuleLoader for PredicateModuleLoader<P, Inner>
where
    P: Fn(Option<&Path>, JsString) -> JsResult<JsString>,
    Inner: ModuleLoader,
{
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        match (self.predicate)(referrer.path(), specifier) {
            Ok(specifier) => {
                self.inner
                    .load_imported_module(referrer, specifier, finish_load, context)
            }
            Err(err) => finish_load(Err(err), context),
        }
    }
}

pub mod predicates {
    use std::path::{Component, Path, PathBuf};

    use boa_engine::{js_string, JsError, JsResult, JsString};

    /// A predicate that only allows loading absolute modules. Will refuse to load
    /// any specifier that starts with `./` or `../`.
    #[inline]
    pub fn absolute_only(_: Option<&Path>, specifier: JsString) -> JsResult<JsString> {
        let specifier = specifier.to_std_string_escaped();
        let short_path = Path::new(&specifier);

        if short_path.starts_with(".") {
            Err(JsError::from_opaque(js_string!("relative path").into()))
        } else {
            Ok(JsString::from(specifier))
        }
    }

    /// A Predicate that can be used with the PrediceModuleLoader that resolves paths
    /// from the referrer and the specifier, normalize the paths and ensure the path
    /// is within a base.
    ///
    /// If the base is empty, that last verification will be skipped. This can be used
    /// for module loaders that don't have an actual filesystem as their backend
    /// (such as [`crate::loaders::HashMapModuleLoader`]).
    #[inline]
    pub fn path_resolver(base: PathBuf) -> impl Fn(Option<&Path>, JsString) -> JsResult<JsString> {
        move |resolver, specifier| {
            let referrer_dir = resolver.and_then(|p| p.parent());
            let specifier = specifier.to_std_string_escaped();
            let short_path = Path::new(&specifier);

            // In ECMAScript, a path is considered relative if it starts with
            // `./` or `../`.
            let long_path = if short_path.starts_with(".") || short_path.starts_with("..") {
                if let Some(r_path) = referrer_dir {
                    base.join(r_path).join(short_path)
                } else {
                    return Err(JsError::from_opaque(
                        js_string!("relative path without referrer").into(),
                    ));
                }
            } else {
                base.join(&specifier)
            };

            if long_path.is_relative() {
                return Err(JsError::from_opaque(
                    js_string!("resolved path is relative").into(),
                ));
            }

            // Normalize the path. We cannot use `canonicalize` here because it will fail
            // if the path doesn't exist.
            let path = long_path
                .components()
                .filter(|c| c != &Component::CurDir || c == &Component::Normal("".as_ref()))
                .try_fold(PathBuf::new(), |mut acc, c| {
                    if c == Component::ParentDir {
                        if acc.as_os_str().is_empty() {
                            return Err(JsError::from_opaque(
                                js_string!("path is outside the module root").into(),
                            ));
                        }
                        acc.pop();
                    } else {
                        acc.push(c);
                    }
                    Ok(acc)
                })?;

            path.strip_prefix(&base)
                .map(|p| JsString::from(p.to_string_lossy().to_string()))
                .map_err(|_| {
                    JsError::from_opaque(js_string!("path is not within the base path").into())
                })
        }
    }
}

#[test]
fn path_resolver_predicate() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;

    let base = PathBuf::from("/base");
    let p = predicates::path_resolver(base.clone());

    let cases = [
        (Some("/hello/base.js"), "file.js", Ok("file.js")),
        (Some("/base/base.js"), "./hello.js", Ok("hello.js")),
        (
            Some("/base/other/base.js"),
            "./hello.js",
            Ok("other/hello.js"),
        ),
        (None, "./hello.js", Err(())),
        (None, "hello.js", Ok("hello.js")),
        (None, "other/hello.js", Ok("other/hello.js")),
        (None, "other/../../hello.js", Err(())),
    ];

    for (referrer, specifier, expected) in cases.into_iter() {
        assert_eq!(
            p(referrer.map(Path::new), JsString::from(specifier)).map_err(|_| ()),
            expected.map(JsString::from)
        );
    }

    Ok(())
}
