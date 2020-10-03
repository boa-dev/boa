//! This module implements the `GcObject` structure.
//!
//! The `GcObject` is a garbage collected Object.

use super::{Object, PROTOTYPE};
use crate::{
    builtins::function::{
        create_unmapped_arguments_object, BuiltInFunction, Function, NativeFunction,
    },
    environment::{
        function_environment_record::BindingStatus, lexical_environment::new_function_environment,
    },
    syntax::ast::node::RcStatementList,
    Context, Executable, Result, Value,
};
use gc::{Finalize, Gc, GcCell, GcCellRef, GcCellRefMut, Trace};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    result::Result as StdResult,
};

/// A wrapper type for an immutably borrowed `Object`.
pub type Ref<'object> = GcCellRef<'object, Object>;

/// A wrapper type for a mutably borrowed `Object`.
pub type RefMut<'object> = GcCellRefMut<'object, Object>;

/// Garbage collected `Object`.
#[derive(Trace, Finalize, Clone, Default)]
pub struct GcObject(Gc<GcCell<Object>>);

// This is needed for the call method since we cannot mutate the function itself since we
// already borrow it so we get the function body clone it then drop the borrow and run the body
enum FunctionBody {
    BuiltIn(NativeFunction),
    Ordinary(RcStatementList),
}

impl GcObject {
    /// Create a new `GcObject` from a `Object`.
    #[inline]
    pub fn new(object: Object) -> Self {
        Self(Gc::new(GcCell::new(object)))
    }

    /// Immutably borrows the `Object`.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    ///# Panics
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_> {
        self.try_borrow().expect("Object already mutably borrowed")
    }

    /// Mutably borrows the Object.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The object cannot be borrowed while this borrow is active.
    ///
    ///# Panics
    /// Panics if the object is currently borrowed.
    #[inline]
    #[track_caller]
    pub fn borrow_mut(&self) -> RefMut<'_> {
        self.try_borrow_mut().expect("Object already borrowed")
    }

    /// Immutably borrows the `Object`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    #[inline]
    pub fn try_borrow(&self) -> StdResult<Ref<'_>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError)
    }

    /// Mutably borrows the object, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The object be borrowed while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    #[inline]
    pub fn try_borrow_mut(&self) -> StdResult<RefMut<'_>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }

    /// Checks if the garbage collected memory is the same.
    #[inline]
    pub fn equals(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::eq(lhs.as_ref(), rhs.as_ref())
    }

    /// Call this object.
    ///
    ///# Panics
    /// Panics if the object is currently mutably borrowed.
    // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
    // <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
    #[track_caller]
    pub fn call(&self, this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let this_function_object = self.clone();
        let f_body = if let Some(function) = self.borrow().as_function() {
            if function.is_callable() {
                match function {
                    Function::BuiltIn(BuiltInFunction(function), _) => {
                        FunctionBody::BuiltIn(*function)
                    }
                    Function::Ordinary {
                        body,
                        params,
                        environment,
                        flags,
                    } => {
                        // Create a new Function environment who's parent is set to the scope of the function declaration (self.environment)
                        // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                        let local_env = new_function_environment(
                            this_function_object,
                            if flags.is_lexical_this_mode() {
                                None
                            } else {
                                Some(this.clone())
                            },
                            Some(environment.clone()),
                            // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                            if flags.is_lexical_this_mode() {
                                BindingStatus::Lexical
                            } else {
                                BindingStatus::Uninitialized
                            },
                        );

                        // Add argument bindings to the function environment
                        for (i, param) in params.iter().enumerate() {
                            // Rest Parameters
                            if param.is_rest_param() {
                                function.add_rest_param(param, i, args, ctx, &local_env);
                                break;
                            }

                            let value = args.get(i).cloned().unwrap_or_else(Value::undefined);
                            function.add_arguments_to_environment(param, value, &local_env);
                        }

                        // Add arguments object
                        let arguments_obj = create_unmapped_arguments_object(args);
                        local_env
                            .borrow_mut()
                            .create_mutable_binding("arguments".to_string(), false);
                        local_env
                            .borrow_mut()
                            .initialize_binding("arguments", arguments_obj);

                        ctx.realm_mut().environment.push(local_env);

                        FunctionBody::Ordinary(body.clone())
                    }
                }
            } else {
                return ctx.throw_type_error("function object is not callable");
            }
        } else {
            return ctx.throw_type_error("not a function");
        };

        match f_body {
            FunctionBody::BuiltIn(func) => func(this, args, ctx),
            FunctionBody::Ordinary(body) => {
                let result = body.run(ctx);
                ctx.realm_mut().environment.pop();

                result
            }
        }
    }

    /// Construct an instance of this object with the specified arguments.
    ///
    ///# Panics
    /// Panics if the object is currently mutably borrowed.
    // <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
    #[track_caller]
    pub fn construct(&self, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let this: Value = Object::create(self.borrow().get(&PROTOTYPE.into())).into();

        let this_function_object = self.clone();
        let body = if let Some(function) = self.borrow().as_function() {
            if function.is_constructable() {
                match function {
                    Function::BuiltIn(BuiltInFunction(function), _) => {
                        FunctionBody::BuiltIn(*function)
                    }
                    Function::Ordinary {
                        body,
                        params,
                        environment,
                        flags,
                    } => {
                        // Create a new Function environment who's parent is set to the scope of the function declaration (self.environment)
                        // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                        let local_env = new_function_environment(
                            this_function_object,
                            Some(this.clone()),
                            Some(environment.clone()),
                            // Arrow functions do not have a this binding https://tc39.es/ecma262/#sec-function-environment-records
                            if flags.is_lexical_this_mode() {
                                BindingStatus::Lexical
                            } else {
                                BindingStatus::Uninitialized
                            },
                        );

                        // Add argument bindings to the function environment
                        for (i, param) in params.iter().enumerate() {
                            // Rest Parameters
                            if param.is_rest_param() {
                                function.add_rest_param(param, i, args, ctx, &local_env);
                                break;
                            }

                            let value = args.get(i).cloned().unwrap_or_else(Value::undefined);
                            function.add_arguments_to_environment(param, value, &local_env);
                        }

                        // Add arguments object
                        let arguments_obj = create_unmapped_arguments_object(args);
                        local_env
                            .borrow_mut()
                            .create_mutable_binding("arguments".to_string(), false);
                        local_env
                            .borrow_mut()
                            .initialize_binding("arguments", arguments_obj);

                        ctx.realm_mut().environment.push(local_env);

                        FunctionBody::Ordinary(body.clone())
                    }
                }
            } else {
                let name = this.get_field("name").display().to_string();
                return ctx.throw_type_error(format!("{} is not a constructor", name));
            }
        } else {
            return ctx.throw_type_error("not a function");
        };

        match body {
            FunctionBody::BuiltIn(function) => {
                function(&this, args, ctx)?;
                Ok(this)
            }
            FunctionBody::Ordinary(body) => {
                let _ = body.run(ctx);

                // local_env gets dropped here, its no longer needed
                let binding = ctx.realm_mut().environment.get_this_binding();
                Ok(binding)
            }
        }
    }
}

impl AsRef<GcCell<Object>> for GcObject {
    #[inline]
    fn as_ref(&self) -> &GcCell<Object> {
        &*self.0
    }
}

/// An error returned by [`GcObject::try_borrow`](struct.GcObject.html#method.try_borrow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already mutably borrowed", f)
    }
}

impl Error for BorrowError {}

/// An error returned by [`GcObject::try_borrow_mut`](struct.GcObject.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;

impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already borrowed", f)
    }
}

impl Error for BorrowMutError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RecursionValueState {
    /// This value is "live": there's an active RecursionLimiter that hasn't been dropped.
    Live,
    /// This value has been seen before, but the recursion limiter has been dropped.
    /// For example:
    /// ```javascript
    /// let b = [];
    /// JSON.stringify([ // Create a recursion limiter for the root here
    ///    b,            // state for b's &GcObject here is None
    ///    b,            // state for b's &GcObject here is Visited
    /// ]);
    /// ```
    Visited,
}

/// Prevents infinite recursion during `Debug::fmt`, `JSON.stringify`, and other conversions.
/// This uses a thread local, so is not safe to use where the object graph will be traversed by
/// multiple threads!
#[derive(Debug)]
pub struct RecursionLimiter {
    /// If this was the first `GcObject` in the tree.
    top_level: bool,
    /// The ptr being kept in the HashSet, so we can delete it when we drop.
    ptr: usize,
    /// If this GcObject has been visited before in the graph, but not in the current branch.
    pub visited: bool,
    /// If this GcObject has been visited in the current branch of the graph.
    pub live: bool,
}

impl Drop for RecursionLimiter {
    fn drop(&mut self) {
        if self.top_level {
            // When the top level of the graph is dropped, we can free the entire map for the next traversal.
            Self::SEEN.with(|hm| hm.borrow_mut().clear());
        } else if !self.live {
            // This was the first RL for this object to become live, so it's no longer live now that it's dropped.
            Self::SEEN.with(|hm| {
                hm.borrow_mut()
                    .insert(self.ptr, RecursionValueState::Visited)
            });
        }
    }
}

impl RecursionLimiter {
    thread_local! {
        /// The map of pointers to `GcObject` that have been visited during the current `Debug::fmt` graph,
        /// and the current state of their RecursionLimiter (dropped or live -- see `RecursionValueState`)
        static SEEN: RefCell<HashMap<usize, RecursionValueState>> = RefCell::new(HashMap::new());
    }

    /// Determines if the specified `GcObject` has been visited, and returns a struct that will free it when dropped.
    ///
    /// This is done by maintaining a thread-local hashset containing the pointers of `GcObject` values that have been
    /// visited. The first `GcObject` visited will clear the hashset, while any others will check if they are contained
    /// by the hashset.
    pub fn new(o: &GcObject) -> Self {
        // We shouldn't have to worry too much about this being moved during Debug::fmt.
        let ptr = (o.as_ref() as *const _) as usize;
        let (top_level, visited, live) = Self::SEEN.with(|hm| {
            let mut hm = hm.borrow_mut();
            let top_level = hm.is_empty();
            let old_state = hm.insert(ptr, RecursionValueState::Live);

            (
                top_level,
                old_state == Some(RecursionValueState::Visited),
                old_state == Some(RecursionValueState::Live),
            )
        });

        Self {
            top_level,
            ptr,
            visited,
            live,
        }
    }
}

impl Debug for GcObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let limiter = RecursionLimiter::new(&self);

        // Typically, using `!limiter.live` would be good enough here.
        // However, the JS object hierarchy involves quite a bit of repitition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, we check if the object has appeared before in the entire graph. This means that objects will appear
        // at most once, hopefully making things a bit clearer.
        if !limiter.visited && !limiter.live {
            f.debug_tuple("GcObject").field(&self.0).finish()
        } else {
            f.write_str("{ ... }")
        }
    }
}
