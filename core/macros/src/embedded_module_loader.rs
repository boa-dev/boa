//! Embedded module loader. Creates a `ModuleLoader` instance that contains
//! files embedded in the binary at build time.

use proc_macro::TokenStream;
use std::path::PathBuf;

use quote::quote;
use syn::{LitInt, LitStr, parse::Parse, Token};

struct EmbedModuleMacroInput {
    path: LitStr,
    max_size: u64,
}

impl Parse for EmbedModuleMacroInput {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let path = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let max_size = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { path, max_size })
    }
}

/// List all the files readable from the given directory, recursively.
fn find_all_files(dir: &mut std::fs::ReadDir, root: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in dir {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();
        if path.is_dir() {
            let Ok(mut sub_dir) = std::fs::read_dir(&path) else {
                continue;
            };
            files.append(&mut find_all_files(&mut sub_dir, root));
        } else if let Ok(path) = path.strip_prefix(root) {
            files.push(path.to_path_buf());
        }
    }
    files
}

/// Implementation of the `embed_module_inner!` macro.
/// This should not be used directly. Use the `embed_module!` macro from the `boa_interop`
/// crate instead.
pub(crate) fn embed_module_impl(input: TokenStream) -> TokenStream {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());

    let input = syn::parse_macro_input!(input as EmbedModuleMacroInput);

    let root = manifest_dir.join(input.path.value());
    let max_size = input.max_size;

    let mut dir = match std::fs::read_dir(root.clone()) {
        Ok(dir) => dir,
        Err(e) => {
            return syn::Error::new_spanned(
                input.path.clone(),
                format!("Error reading directory: {e}"),
            )
            .to_compile_error()
            .into();
        }
    };

    let mut total = 0;
    let files = find_all_files(&mut dir, &root);

    let inner = match files.into_iter().try_fold(quote! {}, |acc, relative_path| {
        let path = root.join(&relative_path);
        let absolute_path = manifest_dir.join(&path).to_string_lossy().to_string();
        let Some(relative_path) = relative_path.to_str() else {
            return Err(syn::Error::new_spanned(
                input.path.clone(),
                "Path has non-Unicode characters",
            ));
        };
        let relative_path = format!("/{}", relative_path.replace(std::path::MAIN_SEPARATOR, "/"));

        // Check the size.
        let size = std::fs::metadata(&path)
            .map_err(|e| {
                syn::Error::new_spanned(input.path.clone(), format!("Error reading file size: {e}"))
            })?
            .len();

        total += size;
        if total > max_size {
            return Err(syn::Error::new_spanned(
                input.path.clone(),
                "Total embedded file size exceeds the maximum size",
            ));
        }

        Ok(quote! {
            #acc

            (
                #relative_path,
                include_bytes!(#absolute_path).as_ref(),
            ),
        })
    }) {
        Ok(inner) => inner,
        Err(e) => return e.to_compile_error().into(),
    };

    let stream = quote! {
        [
            #inner
        ]
    };

    stream.into()
}
