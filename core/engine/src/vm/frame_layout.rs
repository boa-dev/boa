//! Stack and reserved register layout shared by the compiler and runtime.
//!
//! Each frame stores `this` and the function object, followed by its arguments,
//! registers, and temporary stack values. Register indices below are relative to
//! the frame's register pointer. The compiler reserves them before allocating
//! locals and temporaries.

/// Number of stack slots preceding a frame's arguments (`this` and the function).
pub(crate) const FUNCTION_PROLOGUE: u32 = 2;

/// Reserved in every frame and initialized to `undefined` when registers are pushed.
pub(crate) const UNDEFINED_REGISTER_INDEX: usize = 0;

/// Promise capability slots reserved in async functions and async generators.
pub(crate) const PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX: usize = 1;
pub(crate) const PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX: usize = 2;
pub(crate) const PROMISE_CAPABILITY_REJECT_REGISTER_INDEX: usize = 3;

/// Generator object slot reserved in async generators.
pub(crate) const ASYNC_GENERATOR_OBJECT_REGISTER_INDEX: usize = 4;
