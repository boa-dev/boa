//! # Environment Records
//!
//! https://tc39.github.io/ecma262/#sec-environment-records
//! https://tc39.github.io/ecma262/#sec-lexical-environments
//!
//! Some environments are stored as JSObjects. This is for GC, i.e we want to keep an environment if a variable is closed-over (a closure is returned)
//!

use crate::js::value::{Value, ValueData};

struct DeclaritiveEnvironmentRecord {
    env_rec: Value,
}

/// https://tc39.github.io/ecma262/#sec-environment-records
///
/// In the ECMAScript specification Environment Records are hierachical and have a base class with abstract methods.   
/// In this implementation we have a trait which represents the behaviour of all EnvironmentRecord types.
pub trait EnvironmentRecordTrait {
    /// Determine if an Environment Record has a binding for the String value N. Return true if it does and false if it does not.
    fn has_binding(&self, name: String) -> bool;

    /// Create a new but uninitialized mutable binding in an Environment Record. The String value N is the text of the bound name.
    /// If the Boolean argument deletion is true the binding may be subsequently deleted.
    fn create_mutable_binding(&self, name: String, deletion: bool);

    /// Create a new but uninitialized immutable binding in an Environment Record.
    /// The String value N is the text of the bound name.
    /// If strict is true then attempts to set it after it has been initialized will always throw an exception,
    /// regardless of the strict mode setting of operations that reference that binding.
    fn create_immutable_binding(&self, name: String, strict: bool);

    /// Set the value of an already existing but uninitialized binding in an Environment Record.
    /// The String value N is the text of the bound name.
    /// V is the value for the binding and is a value of any ECMAScript language type.
    fn initialize_binding(&self, name: String, value: Value);

    /// Set the value of an already existing mutable binding in an Environment Record.
    /// The String value `name` is the text of the bound name.
    /// value is the `value` for the binding and may be a value of any ECMAScript language type. S is a Boolean flag.
    /// If `strict` is true and the binding cannot be set throw a TypeError exception.
    fn set_mutable_binding(&self, name: String, value: Value, strict: bool);

    /// Returns the value of an already existing binding from an Environment Record.
    /// The String value N is the text of the bound name.
    /// S is used to identify references originating in strict mode code or that
    /// otherwise require strict mode reference semantics.
    fn get_binding_value(&self, name: String, strict: bool) -> Value;

    /// Delete a binding from an Environment Record.
    /// The String value name is the text of the bound name.
    /// If a binding for name exists, remove the binding and return true.
    /// If the binding exists but cannot be removed return false. If the binding does not exist return true.
    fn delete_binding(&self, name: String) -> bool;

    /// Determine if an Environment Record establishes a this binding.
    /// Return true if it does and false if it does not.
    fn has_this_binding(&self) -> bool;

    /// Determine if an Environment Record establishes a super method binding.
    /// Return true if it does and false if it does not.
    fn has_super_binding(&self) -> bool;

    /// If this Environment Record is associated with a with statement, return the with object.
    /// Otherwise, return undefined.
    fn with_base_object(&self) -> Value;
}
