//! Constants for the symbols.

use super::Sym;

impl Sym {
    // Reserved words
    // More information: <https://tc39.es/ecma262/#prod-ReservedWord>

    /// Symbol for the `"await"` string.
    pub const AWAIT: Self = unsafe { Self::new_unchecked(1) };

    /// Symbol for the `"break"` string.
    pub const BREAK: Self = unsafe { Self::new_unchecked(2) };

    /// Symbol for the `"case"` string.
    pub const CASE: Self = unsafe { Self::new_unchecked(3) };

    /// Symbol for the `"catch"` string.
    pub const CATCH: Self = unsafe { Self::new_unchecked(4) };

    /// Symbol for the `"class"` string.
    pub const CLASS: Self = unsafe { Self::new_unchecked(5) };

    /// Symbol for the `"const"` string.
    pub const CONST: Self = unsafe { Self::new_unchecked(6) };

    /// Symbol for the `"continue"` string.
    pub const CONTINUE: Self = unsafe { Self::new_unchecked(7) };

    /// Symbol for the `"debugger"` string.
    pub const DEBUGGER: Self = unsafe { Self::new_unchecked(8) };

    /// Symbol for the `"default"` string.
    pub const DEFAULT: Self = unsafe { Self::new_unchecked(9) };

    /// Symbol for the `"delete"` string.
    pub const DELETE: Self = unsafe { Self::new_unchecked(10) };

    /// Symbol for the `"do"` string.
    pub const DO: Self = unsafe { Self::new_unchecked(11) };

    /// Symbol for the `"else"` string.
    pub const ELSE: Self = unsafe { Self::new_unchecked(12) };

    /// Symbol for the `"enum"` string.
    pub const ENUM: Self = unsafe { Self::new_unchecked(13) };

    /// Symbol for the `"export"` string.
    pub const EXPORT: Self = unsafe { Self::new_unchecked(14) };

    /// Symbol for the `"extends"` string.
    pub const EXTENDS: Self = unsafe { Self::new_unchecked(15) };

    /// Symbol for the `"false"` string.
    pub const FALSE: Self = unsafe { Self::new_unchecked(16) };

    /// Symbol for the `"finally"` string.
    pub const FINALLY: Self = unsafe { Self::new_unchecked(17) };

    /// Symbol for the `"for"` string.
    pub const FOR: Self = unsafe { Self::new_unchecked(18) };

    /// Symbol for the `"function"` string.
    pub const FUNCTION: Self = unsafe { Self::new_unchecked(19) };

    /// Symbol for the `"if"` string.
    pub const IF: Self = unsafe { Self::new_unchecked(20) };

    /// Symbol for the `"import"` string.
    pub const IMPORT: Self = unsafe { Self::new_unchecked(21) };

    /// Symbol for the `"in"` string.
    pub const IN: Self = unsafe { Self::new_unchecked(22) };

    /// Symbol for the `"instanceof"` string.
    pub const INSTANCEOF: Self = unsafe { Self::new_unchecked(23) };

    /// Symbol for the `"new"` string.
    pub const NEW: Self = unsafe { Self::new_unchecked(24) };

    /// Symbol for the `"null"` string.
    pub const NULL: Self = unsafe { Self::new_unchecked(25) };

    /// Symbol for the `"return"` string.
    pub const RETURN: Self = unsafe { Self::new_unchecked(26) };

    /// Symbol for the `"super"` string.
    pub const SUPER: Self = unsafe { Self::new_unchecked(27) };

    /// Symbol for the `"switch"` string.
    pub const SWITCH: Self = unsafe { Self::new_unchecked(28) };

    /// Symbol for the `"this"` string.
    pub const THIS: Self = unsafe { Self::new_unchecked(29) };

    /// Symbol for the `"throw"` string.
    pub const THROW: Self = unsafe { Self::new_unchecked(30) };

    /// Symbol for the `"true"` string.
    pub const TRUE: Self = unsafe { Self::new_unchecked(31) };

    /// Symbol for the `"try"` string.
    pub const TRY: Self = unsafe { Self::new_unchecked(32) };

    /// Symbol for the `"typeof"` string.
    pub const TYPEOF: Self = unsafe { Self::new_unchecked(33) };

    /// Symbol for the `"var"` string.
    pub const VAR: Self = unsafe { Self::new_unchecked(34) };

    /// Symbol for the `"void"` string.
    pub const VOID: Self = unsafe { Self::new_unchecked(35) };

    /// Symbol for the `"while"` string.
    pub const WHILE: Self = unsafe { Self::new_unchecked(36) };

    /// Symbol for the `"with"` string.
    pub const WITH: Self = unsafe { Self::new_unchecked(37) };

    /// Symbol for the `"yield"` string.
    pub const YIELD: Self = unsafe { Self::new_unchecked(38) };

    /// Symbol for the empty string (`""`).
    pub const EMPTY_STRING: Self = unsafe { Self::new_unchecked(39) };

    /// Symbol for the `"arguments"` string.
    pub const ARGUMENTS: Self = unsafe { Self::new_unchecked(40) };

    /// Symbol for the `"eval"` string.
    pub const EVAL: Self = unsafe { Self::new_unchecked(41) };

    /// Symbol for the `"RegExp"` string.
    pub const REGEXP: Self = unsafe { Self::new_unchecked(42) };

    /// Symbol for the `"get"` string.
    pub const GET: Self = unsafe { Self::new_unchecked(43) };

    /// Symbol for the `"set"` string.
    pub const SET: Self = unsafe { Self::new_unchecked(44) };

    /// Symbol for the `"<main>"` string.
    pub const MAIN: Self = unsafe { Self::new_unchecked(45) };

    /// Symbol for the `"raw"` string.
    pub const RAW: Self = unsafe { Self::new_unchecked(46) };

    /// Symbol for the `"static"` string.
    pub const STATIC: Self = unsafe { Self::new_unchecked(47) };

    /// Symbol for the `"prototype"` string.
    pub const PROTOTYPE: Self = unsafe { Self::new_unchecked(48) };

    /// Symbol for the `"constructor"` string.
    pub const CONSTRUCTOR: Self = unsafe { Self::new_unchecked(49) };

    /// Symbol for the `"implements"` string.
    pub const IMPLEMENTS: Self = unsafe { Self::new_unchecked(50) };

    /// Symbol for the `"interface"` string.
    pub const INTERFACE: Self = unsafe { Self::new_unchecked(51) };

    /// Symbol for the `"let"` string.
    pub const LET: Self = unsafe { Self::new_unchecked(52) };

    /// Symbol for the `"package"` string.
    pub const PACKAGE: Self = unsafe { Self::new_unchecked(53) };

    /// Symbol for the `"private"` string.
    pub const PRIVATE: Self = unsafe { Self::new_unchecked(54) };

    /// Symbol for the `"protected"` string.
    pub const PROTECTED: Self = unsafe { Self::new_unchecked(55) };

    /// Symbol for the `"public"` string.
    pub const PUBLIC: Self = unsafe { Self::new_unchecked(56) };

    /// Symbol for the `"anonymous"` string.
    pub const ANONYMOUS: Self = unsafe { Self::new_unchecked(57) };

    /// Symbol for the `"async"` string.
    pub const ASYNC: Self = unsafe { Self::new_unchecked(58) };

    /// Symbol for the `"of"` string.
    pub const OF: Self = unsafe { Self::new_unchecked(59) };

    /// Symbol for the `"target"` string.
    pub const TARGET: Self = unsafe { Self::new_unchecked(60) };

    /// Symbol for the `"as"` string.
    pub const AS: Self = unsafe { Self::new_unchecked(61) };

    /// Symbol for the `"from"` string.
    pub const FROM: Self = unsafe { Self::new_unchecked(62) };

    /// Symbol for the `"__proto__"` string.
    pub const __PROTO__: Self = unsafe { Self::new_unchecked(63) };
}
