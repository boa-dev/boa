//! Reflection objects for inspecting debuggee state
//!
//! This module provides safe wrapper objects for inspecting the state of
//! the debuggee (the JavaScript code being debugged) without directly
//! exposing VM internals.

use crate::{
    Context, JsObject, JsResult, JsString, JsValue,
    vm::{CallFrame, CodeBlock},
};
use boa_ast::Position;
use boa_gc::Gc;
use std::fmt;

/// A reflection object representing a call frame in the debuggee
///
/// This provides safe access to frame information without exposing
/// the raw `CallFrame` structure.
#[derive(Debug, Clone)]
pub struct DebuggerFrame {
    /// The function name
    pub function_name: JsString,

    /// The source path
    pub source_path: String,

    /// The current position in the source
    pub position: Option<Position>,

    /// The program counter (bytecode offset)
    pub pc: u32,

    /// The frame depth (0 = top-level)
    pub depth: usize,

    /// Reference to the code block (for internal use)
    code_block: Gc<CodeBlock>,
}

impl DebuggerFrame {
    /// Creates a new `DebuggerFrame` from a `CallFrame`
    #[must_use]
    pub fn from_call_frame(frame: &CallFrame, depth: usize) -> Self {
        let location = frame.position();
        Self {
            function_name: location.function_name,
            source_path: location.path.to_string(),
            position: location.position,
            pc: frame.pc,
            depth,
            code_block: frame.code_block().clone(),
        }
    }

    /// Gets the function name
    #[must_use]
    pub fn function_name(&self) -> &JsString {
        &self.function_name
    }

    /// Gets the source file path
    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Gets the line number (1-based) if available
    #[must_use]
    pub fn line_number(&self) -> Option<u32> {
        self.position.map(Position::line_number)
    }

    /// Gets the column number (1-based) if available
    #[must_use]
    pub fn column_number(&self) -> Option<u32> {
        self.position.map(Position::column_number)
    }

    /// Gets the program counter (bytecode offset)
    #[must_use]
    pub fn pc(&self) -> u32 {
        self.pc
    }

    /// Gets the frame depth (0 = top-level, 1 = first call, etc.)
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Gets the code block for this frame
    #[must_use]
    pub fn code_block(&self) -> &Gc<CodeBlock> {
        &self.code_block
    }

    /// Evaluates an expression in the context of this frame
    ///
    /// This can be used to inspect variables, evaluate watch expressions, etc.
    pub fn eval(&self, _context: &mut Context, _expression: &str) -> JsResult<JsValue> {
        // TODO(al): Implement expression evaluation in frame context
        // This requires access to the frame's environment and scope chain
        Err(crate::JsNativeError::error()
            .with_message("Frame evaluation not yet implemented")
            .into())
    }
}

impl fmt::Display for DebuggerFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}",
            self.function_name.to_std_string_escaped(),
            self.source_path
        )?;
        if let Some(pos) = self.position {
            write!(f, ":{}:{}", pos.line_number(), pos.column_number())?;
        }
        Ok(())
    }
}

/// A reflection object representing a script or code block
///
/// This provides information about compiled code without exposing
/// internal VM structures.
#[derive(Debug, Clone)]
pub struct DebuggerScript {
    /// Unique identifier for this script
    pub id: super::ScriptId,

    /// The source code
    pub source: Option<String>,

    /// The source file path
    pub path: String,

    /// The function/script name
    pub name: JsString,
}

impl DebuggerScript {
    /// Creates a new `DebuggerScript`
    #[must_use]
    pub fn new(id: super::ScriptId, path: String, name: JsString) -> Self {
        Self {
            id,
            source: None,
            path,
            name,
        }
    }

    /// Creates a `DebuggerScript` with source code
    #[must_use]
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Gets the script ID
    #[must_use]
    pub fn id(&self) -> super::ScriptId {
        self.id
    }

    /// Gets the script name
    #[must_use]
    pub fn name(&self) -> &JsString {
        &self.name
    }

    /// Gets the source file path
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Gets the source code if available
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Gets the number of lines in the source code
    #[must_use]
    pub fn line_count(&self) -> Option<usize> {
        self.source.as_ref().map(|s| s.lines().count())
    }

    /// Gets a specific line from the source code (0-based)
    #[must_use]
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.source.as_ref()?.lines().nth(line)
    }
}

impl fmt::Display for DebuggerScript {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name.to_std_string_escaped(), self.path)
    }
}

/// A reflection object representing an object in the debuggee
///
/// This provides safe access to object properties and methods without
/// directly exposing the `JsObject`.
#[derive(Debug, Clone)]
pub struct DebuggerObject {
    /// The wrapped object
    object: JsObject,

    /// A preview/summary of the object for display
    preview: String,
}

impl DebuggerObject {
    /// Creates a new `DebuggerObject` from a `JsObject`
    pub fn from_object(object: JsObject, context: &mut Context) -> Self {
        // Generate a preview string for the object
        let preview = Self::generate_preview(&object, context);

        Self { object, preview }
    }

    /// Generates a preview string for an object
    fn generate_preview(object: &JsObject, _context: &mut Context) -> String {
        // TODO(al): Implement better object preview generation
        // For now, just show the object type
        let proto = object
            .borrow()
            .prototype()
            .map_or_else(|| "null".to_string(), |p| format!("{p:?}"));
        format!("[object {proto}]")
    }

    /// Gets the preview string for this object
    #[must_use]
    pub fn preview(&self) -> &str {
        &self.preview
    }

    /// Gets the wrapped `JsObject`
    ///
    /// Note: This exposes the internal object and should be used carefully
    pub fn as_js_object(&self) -> &JsObject {
        &self.object
    }

    /// Gets a property from the object
    pub fn get_property(&self, key: &str, context: &mut Context) -> JsResult<JsValue> {
        self.object.get(JsString::from(key), context)
    }

    /// Gets all own property names
    pub fn own_property_names(&self, context: &mut Context) -> JsResult<Vec<JsString>> {
        // TODO(al): Implement proper property enumeration
        let _ = context;
        Ok(vec![])
    }

    /// Gets the prototype of this object
    pub fn prototype(&self, context: &mut Context) -> JsResult<Option<DebuggerObject>> {
        let _ = context;
        if let Some(proto) = self.object.prototype() {
            Ok(Some(DebuggerObject::from_object(proto, context)))
        } else {
            Ok(None)
        }
    }

    /// Checks if this object is callable (a function)
    pub fn is_callable(&self) -> bool {
        self.object.is_callable()
    }

    /// Checks if this object is a constructor
    pub fn is_constructor(&self) -> bool {
        self.object.is_constructor()
    }
}

impl fmt::Display for DebuggerObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.preview)
    }
}
