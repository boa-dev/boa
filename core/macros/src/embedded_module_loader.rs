//! Embedded module loader. Creates a `ModuleLoader` instance that contains
//! files embedded in the binary at build time.

use proc_macro::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};
use std::fs;
use std::path::PathBuf;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, LitStr, Token, parse::Parse};

#[derive(Copy, Clone)]
enum CompressType {
    None,

    #[cfg(feature = "embedded_lz4")]
    Lz4,
}

impl ToTokens for CompressType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CompressType::None => tokens.append(proc_macro2::Literal::string("none")),
            #[cfg(feature = "embedded_lz4")]
            CompressType::Lz4 => tokens.append(proc_macro2::Literal::string("lz4")),
        }
    }
}

impl Parse for CompressType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ty = input.parse::<LitStr>()?;
        match ty.value().as_ref() {
            "none" => Ok(Self::None),

            #[cfg(feature = "lz4_flex")]
            "lz4" => Ok(Self::Lz4),

            other => Err(input.error(format!("Invalid compression type: {other}"))),
        }
    }
}

enum Argument {
    Path(LitStr),
    MaxSize(u64),
    Compress(CompressType),
}

impl Parse for Argument {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        // If it's a single literal string, use it as path.
        if lookahead.peek(LitStr) {
            return Ok(Self::Path(input.parse()?));
        }

        match input.parse::<Ident>()?.to_string().as_ref() {
            "path" => {
                let _sep = input.parse::<Token![,]>()?;
                Ok(Self::Path(input.parse()?))
            }
            "max_size" => {
                let _sep = input.parse::<Token![,]>()?;
                let value: LitInt = input.parse()?;
                Ok(Self::MaxSize(value.base10_parse()?))
            }
            "compress" => {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![=]) {
                    let _sep = input.parse::<Token![=]>()?;
                    Ok(Self::Compress(input.parse()?))
                } else {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "embedded_lz4")] {
                            Ok(Self::Compress(CompressType::Lz4))
                        } else {
                            Err(input.error("No compression available by default."))
                        }
                    }
                }
            }
            other => Err(input.error(format!("Invalid argument name: {other}"))),
        }
    }
}

struct EmbedModuleMacroInput {
    path: LitStr,
    max_size: Option<u64>,
    compress: CompressType,
}

impl Parse for EmbedModuleMacroInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let tokens = input.fork();
        let arguments = Punctuated::<Argument, Token![,]>::parse_terminated(input)?;

        let mut path = None;
        let mut max_size = None;
        let mut compress = CompressType::None;
        for arg in arguments {
            match arg {
                Argument::Path(p) => path = Some(p),
                Argument::MaxSize(sz) => max_size = Some(sz),
                Argument::Compress(t) => compress = t,
            }
        }

        if let Some(path) = path {
            Ok(Self {
                path,
                max_size,
                compress,
            })
        } else {
            Err(tokens.error("Must specify a path."))
        }
    }
}

/// List all the files readable from the given directory, recursively.
fn find_all_files(dir: &mut fs::ReadDir, root: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in dir {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();
        if path.is_dir() {
            let Ok(mut sub_dir) = fs::read_dir(&path) else {
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
    let max_size = input.max_size.unwrap_or(u64::MAX);

    let mut dir = match fs::read_dir(root.clone()) {
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
        let relative_path = format!("{}{}", std::path::MAIN_SEPARATOR, relative_path);

        // Check the size.
        let size = fs::metadata(&path)
            .map_err(|e| {
                syn::Error::new_spanned(input.path.clone(), format!("Error reading file size: {e}"))
            })?
            .len();

        total += size;
        if total > max_size {
            return Err(syn::Error::new_spanned(
                input.path.clone(),
                "The total embedded file size exceeds the maximum size",
            ));
        }

        let bytes = fs::read_to_string(&absolute_path).map_err(|e| {
            syn::Error::new_spanned(
                input.path.clone(),
                format!("Could not read {absolute_path}: {e}"),
            )
        })?;

        let compress = input.compress;
        let bytes = match compress {
            CompressType::None => bytes.as_bytes().to_vec(),

            #[cfg(feature = "embedded_lz4")]
            CompressType::Lz4 => lz4_flex::compress_prepend_size(bytes.as_bytes()),
        };

        Ok(quote! {
            #acc

            (
                #compress,
                #relative_path,
                &[#(#bytes),*] as &[u8],
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
