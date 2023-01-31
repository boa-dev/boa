use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

/// A source of ECMAScript code.
///
/// [`Source`]s can be created from plain [`str`]s, file [`Path`]s or more generally, any [`Read`]
/// instance.
#[derive(Debug)]
pub struct Source<'path, R> {
    pub(crate) reader: R,
    pub(crate) path: Option<&'path Path>,
}

impl<'bytes> Source<'static, &'bytes [u8]> {
    /// Creates a new `Source` from any type equivalent to a slice of bytes e.g. [`&str`][str],
    /// <code>[Vec]<[u8]></code>, <code>[Box]<[\[u8\]][slice]></code> or a plain slice
    /// <code>[&\[u8\]][slice]</code>.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_parser::Source;
    /// let code = r#"var array = [5, 4, 3, 2, 1];"#;
    /// let source = Source::from_bytes(code);
    /// ```
    ///
    /// [slice]: std::slice
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(source: &'bytes T) -> Self {
        Self {
            reader: source.as_ref(),
            path: None,
        }
    }
}

impl<'path> Source<'path, BufReader<File>> {
    /// Creates a new `Source` from a `Path` to a file.
    ///
    /// # Errors
    ///
    /// See [`File::open`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use boa_parser::Source;
    /// # use std::{fs::File, path::Path};
    /// # fn main() -> std::io::Result<()> {
    /// let path = Path::new("script.js");
    /// let source = Source::from_filepath(path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_filepath(source: &'path Path) -> io::Result<Self> {
        let reader = File::open(source)?;
        Ok(Self {
            reader: BufReader::new(reader),
            path: Some(source),
        })
    }
}

impl<'path, R: Read> Source<'path, R> {
    /// Creates a new `Source` from a [`Read`] instance and an optional [`Path`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use boa_parser::Source;
    /// # use std::{fs::File, io::Read, path::Path};
    /// # fn main() -> std::io::Result<()> {
    /// let strictler = r#""use strict""#;
    ///
    /// let path = Path::new("no_strict.js");
    /// let file = File::open(path)?;
    /// let strict = strictler.as_bytes().chain(file);
    ///
    /// let source = Source::from_reader(strict, Some(path));
    /// #    Ok(())
    /// # }
    /// ```
    pub const fn from_reader(reader: R, path: Option<&'path Path>) -> Self {
        Self { reader, path }
    }
}
