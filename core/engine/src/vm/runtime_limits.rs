/// Default maximum number of bytes a JS-controlled `Vec` reserve may request.
///
/// Matches the default `ArrayBuffer` host hook (`1_610_612_736` bytes).
pub const DEFAULT_MAX_ALLOCATION: usize = 1_610_612_736;

/// Represents the limits of different runtime operations.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeLimits {
    /// Max stack size before an error is thrown.
    stack_size: usize,

    /// Max loop iterations before an error is thrown.
    loop_iteration: u64,

    /// Max backtrace count in exception.
    backtrace_limit: usize,

    /// Max function recursion limit
    recursion: usize,

    /// Max number of bytes a JS-controlled `Vec` reserve may request.
    max_allocation: usize,
}

impl Default for RuntimeLimits {
    #[inline]
    fn default() -> Self {
        Self {
            loop_iteration: u64::MAX,
            recursion: 512,
            backtrace_limit: 50,
            stack_size: 1024 * 10,
            max_allocation: DEFAULT_MAX_ALLOCATION,
        }
    }
}

impl RuntimeLimits {
    /// Return the loop iteration limit.
    ///
    /// If the limit is exceeded in a loop it will throw an error.
    ///
    /// The limit value [`u64::MAX`] means that there is no limit.
    #[inline]
    #[must_use]
    pub const fn loop_iteration_limit(&self) -> u64 {
        self.loop_iteration
    }

    /// Set the loop iteration limit.
    ///
    /// If the limit is exceeded in a loop it will throw an error.
    ///
    /// Setting the limit to [`u64::MAX`] means that there is no limit.
    #[inline]
    pub fn set_loop_iteration_limit(&mut self, value: u64) {
        self.loop_iteration = value;
    }

    /// Disable loop iteration limit.
    #[inline]
    pub fn disable_loop_iteration_limit(&mut self) {
        self.loop_iteration = u64::MAX;
    }

    /// Get max backtrace limit for an exception.
    ///
    /// Default is 50.
    #[inline]
    #[must_use]
    pub const fn backtrace_limit(&self) -> usize {
        self.backtrace_limit
    }

    /// Set max backtrace limit for an exception.
    #[inline]
    pub fn set_backtrace_limit(&mut self, value: usize) {
        self.backtrace_limit = value;
    }

    /// Get max stack size.
    #[inline]
    #[must_use]
    pub const fn stack_size_limit(&self) -> usize {
        self.stack_size
    }

    /// Set max stack size before an error is thrown.
    #[inline]
    pub fn set_stack_size_limit(&mut self, value: usize) {
        self.stack_size = value;
    }

    /// Get recursion limit.
    #[inline]
    #[must_use]
    pub const fn recursion_limit(&self) -> usize {
        self.recursion
    }

    /// Set recursion limit before an error is thrown.
    #[inline]
    pub fn set_recursion_limit(&mut self, value: usize) {
        self.recursion = value;
    }

    /// Get the maximum number of bytes a JS-controlled `Vec` reserve may request.
    ///
    /// [`usize::MAX`] means there is no extra cap; `try_reserve_exact` is still used
    /// so allocator failure becomes an error instead of aborting the process.
    #[inline]
    #[must_use]
    pub const fn max_allocation(&self) -> usize {
        self.max_allocation
    }

    /// Set the maximum number of bytes a JS-controlled `Vec` reserve may request.
    ///
    /// Setting the limit to [`usize::MAX`] means that there is no extra cap.
    #[inline]
    pub fn set_max_allocation(&mut self, value: usize) {
        self.max_allocation = value;
    }

    /// Disable the extra allocation cap.
    ///
    /// Allocator failure is still mapped to an error via `try_reserve_exact`.
    #[inline]
    pub fn disable_max_allocation(&mut self) {
        self.max_allocation = usize::MAX;
    }
}
