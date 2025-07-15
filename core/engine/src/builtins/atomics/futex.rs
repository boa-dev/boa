// TODO: track https://github.com/rust-lang/rfcs/pull/3467 to see if we can use `UnsafeAliased` instead
// of raw pointers.

// A bit of context about how exactly this thing works.
//
// `Atomics.wait/notify` is basically an emulation of the "futex" syscall, which internally uses
// a wait queue attached to a certain memory address, where processes and threads can manipulate
// it to synchronize between them.
// More information: https://en.wikipedia.org/wiki/Futex
//
// Our emulation of the API is composed by three components:
//
// - `FutexWaiters`, which is a map of addresses to the corresponding wait queue for that address.
//   Internally uses intrusive linked lists to avoid allocating when adding a new waiter, which
//   reduces the time spent by a thread in the critical section.
//
// - `FutexWaiter`, which contains all the data necessary to be able to wake a waiter from another
//   thread. It also contains a `waiting` boolean, that is checked after waking up to see
//   if the waiter was indeed woken up or if it just sporadically woke up (yes, this is a thing that
//   can happen per the documentation of `CondVar`).
//
// - `CRITICAL_SECTION`, a global static that must be locked before registering or notifying any
//   waiter. This guarantees that only one agent can write to the wait queues at any point in time.
//
// We can emulate a typical execution using the API for demonstration purposes.
// At the start of the program, we initially have an empty map of wait queues. We represent this
// graphically as:
//
//   Address   │
//             │
// ────────────┼────────────────────────────────────────────────────────────────────
//             │
//             │
//   <empty>   │
//             │
//             │
//
// Each row here will represent an address and the corresponding wait queue for that address.
//
// Let's suppose that "Thread 2" wants to wait on the address 50. After locking the global mutex,
// it first creates a new instante of a `FutexWaiter` and passes a pointer to it to the
// `FutexWaiters::add_waiter`:
//
//   Address   │
//             │
// ────────────┼──────────────────────────────────────────────────────────────────────
//             │
//             │       ┌───────────────┐
//             │    ┌─►│               │
//             │    │  │ Thread 2      │
//             │    │  │ FutexWaiter   │
//     50      ├────┘  │               │
//             │       │               │
//             │       │ cond_var      │
//             │       │ waiting: true │
//             │       │               │
//             │       └───────────────┘
//             │
//
// Immediately after this, "Thread 2" calls `cond_var.wait`, unlocks the global mutex and sleeps
// until it is notified again (ignoring the spurious wakeups, those are handled in an infinite loop
// anyways).
//
// Now, let's suppose that `Thread 1` has now acquired the lock and now wants to also
// wait on the address `50`. Doing the same procedure as "Thread 2", our map now looks like:
//
//   Address   │
//             │
// ────────────┼──────────────────────────────────────────────────────────────────────
//             │
//             │       ┌───────────────┐        ┌───────────────┐
//             │    ┌─►│               ├───────►│               │
//             │    │  │ Thread 2      │        │ Thread 1      │
//             │    │  │ FutexWaiter   │        │ FutexWaiter   │
//     50      ├────┘  │               │        │               │
//             │       │               │        │               │
//             │       │ cond_var      │        │ cond_var      │
//             │       │ waiting: true │◄───────┤ waiting: true │
//             │       │               │        │               │
//             │       └───────────────┘        └───────────────┘
//             │
//
// Note how the head of our list contains the first waiter which was registered, and the
// tail of our list is our most recent waiter.
//
// After "Thread 1" sleeps, "Thread 3" has the opportunity to lock the global mutex.
// In this case, "Thread 3" will notify one waiter of the address 50 using the `cond_var` inside
// `FutexWaiter`, and will also remove it from the linked list. In this case
// the notified thread is "Thread 2":
//
//   Address   │
//             │
// ────────────┼──────────────────────────────────────────────────────────────────────
//             │
//             │       ┌────────────────┐       ┌────────────────┐
//             │       │                │   ┌──►│                │
//             │       │ Thread 2       │   │   │ Thread 1       │
//             │       │ FutexWaiter    │   │   │ FutexWaiter    │
//     50      ├───┐   │                │   │   │                │
//             │   │   │                │   │   │                │
//             │   │   │ cond_var       │   │   │ cond_var       │
//             │   │   │ waiting: false │   │   │ waiting: true  │
//             │   │   │                │   │   │                │
//             │   │   └────────────────┘   │   └────────────────┘
//             │   │                        │
//             │   └────────────────────────┘
//             │
//
// Then, when the lock is released and "Thread 2" has woken up, it tries to lock the global mutex
// again, checking if `waiting` is true to manually remove itself from the queue if that's the case.
// In this case, `waiting` is false, which doesn't require any other handling, so it just
// removes the `FutexWaiter` from its stack and returns `AtomicsWaitResult::Ok`.
//
//   Address   │
//             │
// ────────────┼──────────────────────────────────────────────────────────────────────
//             │
//             │                                ┌────────────────┐
//             │    ┌──────────────────────────►│                │
//             │    │                           │ Thread 1       │
//             │    │                           │ FutexWaiter    │
//     50      ├────┘                           │                │
//             │                                │                │
//             │                                │ cond_var       │
//             │                                │ waiting: true  │
//             │                                │                │
//             │                                └────────────────┘
//             │
//             │
//             │
//
// In a future point in time, "Thread 1" will be notified, which will proceed with the
// exact same steps as "Thread 2", emptying the wait queue and finishing the execution of our
// program.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::expl_impl_clone_on_copy)]

use std::{
    ptr,
    sync::{Arc, atomic::Ordering},
};

use crate::{
    Context, JsNativeError, JsResult, JsValue,
    builtins::{
        array_buffer::{SharedArrayBuffer, utils::SliceRef},
        promise::ResolvingFunctions,
        typed_array::Element,
    },
    job::{GenericJob, TimeoutJob},
    js_string,
    sys::time::{Duration, Instant},
};

use std::sync::{Condvar, Mutex, MutexGuard};

use boa_string::JsString;
use intrusive_collections::{LinkedList, LinkedListLink, UnsafeRef, intrusive_adapter};
use portable_atomic::AtomicBool;
use rustc_hash::FxHashMap;
use small_btree::{Entry, SmallBTreeMap};

/// The result of the [`wait`] and [`wait_async`] functions.
#[derive(Debug, Clone, Copy)]
pub(super) enum AtomicsWaitResult {
    NotEqual,
    TimedOut,
    Ok,
}

impl AtomicsWaitResult {
    pub(super) fn to_js_string(self) -> JsString {
        match self {
            AtomicsWaitResult::NotEqual => js_string!("not-equal"),
            AtomicsWaitResult::TimedOut => js_string!("timed-out"),
            AtomicsWaitResult::Ok => js_string!("ok"),
        }
    }
}

/// Data used by async waiters.
#[derive(Debug)]
struct AsyncWaiterData {
    // Only here to ensure the buffer does not get collected before all its
    // waiters.
    _buffer: SharedArrayBuffer,
    notified: AtomicBool,
}

/// A waiter of a memory address.
#[derive(Debug)]
struct FutexWaiter {
    link: LinkedListLink,
    cond_var: Condvar,
    addr: usize,
    async_data: Option<AsyncWaiterData>,
}

intrusive_adapter!(FutexWaiterAdapter = UnsafeRef<FutexWaiter>: FutexWaiter { link: LinkedListLink });

impl FutexWaiter {
    /// Creates a new `FutexWaiter` that will block the current thread while waiting.
    fn new_sync(addr: usize) -> Self {
        Self {
            link: LinkedListLink::new(),
            cond_var: Condvar::new(),
            addr,
            async_data: None,
        }
    }

    /// Creates a new `FutexWaiter` that will NOT block the current thread while waiting.
    #[allow(
        clippy::arc_with_non_send_sync,
        reason = "across threads we only access the fields that are `Sync`"
    )]
    fn new_async(buffer: SharedArrayBuffer, addr: usize) -> Arc<FutexWaiter> {
        Arc::new(Self {
            link: LinkedListLink::new(),
            cond_var: Condvar::new(),
            addr,
            async_data: Some(AsyncWaiterData {
                _buffer: buffer,
                notified: AtomicBool::new(false),
            }),
        })
    }
}

/// An async waiter of a memory address.
#[derive(Clone, Debug)]
struct AsyncFutexWaiter(Arc<FutexWaiter>);

impl std::hash::Hash for AsyncFutexWaiter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).addr().hash(state);
    }
}

impl PartialEq for AsyncFutexWaiter {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for AsyncFutexWaiter {}

#[derive(Debug, Default)]
pub(crate) struct AsyncPendingWaiters {
    waiters: FxHashMap<AsyncFutexWaiter, ResolvingFunctions>,
}

impl AsyncPendingWaiters {
    /// Creates a new `AsyncPendingWaiters`.
    pub(crate) fn new() -> Self {
        Self {
            waiters: FxHashMap::default(),
        }
    }

    fn insert(&mut self, waiter: AsyncFutexWaiter, cap: ResolvingFunctions) {
        // Should have been added to the waiters list at this point.
        debug_assert!(waiter.0.link.is_linked());
        self.waiters.insert(waiter, cap);
    }

    fn remove(&mut self, waiter: &AsyncFutexWaiter) -> Option<ResolvingFunctions> {
        self.waiters.remove(waiter)
    }

    /// Gets the number of waiters in the list of pending waiters.
    pub(crate) fn len(&self) -> usize {
        self.waiters.len()
    }

    /// Gets all waiters that have been notified and enqueues their corresponding
    /// generic jobs.
    ///
    /// This is roughly equivalent to
    /// [`EnqueueResolveInAgentJob ( agentSignifier, promiseCapability, resolution )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-enqueueresolveinagentjob
    pub(crate) fn enqueue_waiter_jobs(&mut self, context: &mut Context) {
        for (waiter, functions) in self.waiters.extract_if(|k, _| {
            k.0.async_data
                .as_ref()
                .is_some_and(|data| data.notified.load(Ordering::Relaxed))
        }) {
            // Should have been removed from the waiters list at this point.
            debug_assert!(!waiter.0.link.is_linked());
            let realm = functions
                .resolve
                .get_function_realm(context)
                .expect("cannot fail for the default resolving functions");
            context.enqueue_job(
                GenericJob::new(
                    move |context| {
                        functions.resolve.call(
                            &JsValue::undefined(),
                            &[AtomicsWaitResult::Ok.to_js_string().into()],
                            context,
                        )
                    },
                    realm,
                )
                .into(),
            );
        }
    }
}

impl Drop for AsyncPendingWaiters {
    /// If the parent `Context` gets dropped, this will automatically
    /// remove all active waiters from the waiters list.
    fn drop(&mut self) {
        if self.waiters.is_empty() {
            return;
        }

        let Ok(mut global_waiters) = FutexWaiters::get() else {
            // Cannot cleanup a poisoned mutex.
            return;
        };

        for (waiter, _) in self.waiters.drain() {
            if waiter.0.link.is_linked() {
                // SAFETY: All waiters in the pending waiters map must be valid.
                unsafe {
                    global_waiters.remove_waiter(&waiter.0);
                }
            }
        }
    }
}

/// List of memory addresses and its corresponding list of waiters for that address.
#[derive(Debug)]
struct FutexWaiters {
    waiters: SmallBTreeMap<usize, LinkedList<FutexWaiterAdapter>, 16>,
}

// SAFETY: `FutexWaiters` is not constructable outside its `get` method, and it's only exposed by
//          a global lock, meaning the inner data of `FutexWaiters` (which includes non-Send pointers)
//          can only be accessed by a single thread at once.
unsafe impl Send for FutexWaiters {}

impl FutexWaiters {
    /// Gets the map of all shared data addresses and its corresponding list of agents waiting on that location.
    fn get() -> JsResult<MutexGuard<'static, Self>> {
        static CRITICAL_SECTION: Mutex<FutexWaiters> = Mutex::new(FutexWaiters {
            waiters: SmallBTreeMap::new(),
        });

        CRITICAL_SECTION.lock().map_err(|_| {
            JsNativeError::typ()
                .with_message("failed to synchronize with the agent cluster")
                .into()
        })
    }

    /// Notifies at most `max_count` waiters that are waiting on the address `addr`, and
    /// returns the number of waiters that were notified.
    ///
    /// Equivalent to [`RemoveWaiters`][remove] and [`NotifyWaiter`][notify], but in a single operation.
    ///
    /// [remove]: https://tc39.es/ecma262/#sec-removewaiters
    /// [notify]: https://tc39.es/ecma262/#sec-notifywaiter
    fn notify_many(&mut self, addr: usize, max_count: u64, context: &mut Context) -> u64 {
        let Entry::Occupied(mut wl) = self.waiters.entry(addr) else {
            return 0;
        };

        for i in 0..max_count {
            let Some(elem) = wl.get_mut().pop_front() else {
                wl.remove();
                return i;
            };

            elem.cond_var.notify_one();

            if let Some(async_data) = &elem.async_data {
                async_data.notified.store(true, Ordering::Relaxed);

                // SAFETY: All entries on the waiter list must be valid, and all async entries
                // must come from an Arc.
                let elem = unsafe {
                    AsyncFutexWaiter(Arc::from_raw(UnsafeRef::into_raw(elem).cast_const()))
                };

                if let Some(functions) = context.pending_waiters.remove(&elem) {
                    // The waiter was part of this context, so we can safely resolve the promise from here.
                    functions
                        .resolve
                        .call(
                            &JsValue::undefined(),
                            &[AtomicsWaitResult::Ok.to_js_string().into()],
                            context,
                        )
                        .expect("cannot fail per the spec");
                }
            }
        }

        if wl.get().is_empty() {
            wl.remove();
        }

        max_count
    }

    /// # Safety
    ///
    /// - `node` must NOT be linked to an existing waiter list.
    /// - `node` must always reference a valid instance of `FutexWaiter` until `node` is
    ///   removed from its linked list. This can happen by either `remove_waiter` or `notify_many`.
    unsafe fn add_waiter(&mut self, node: &FutexWaiter) {
        // SAFETY: `node` must point to a valid instance.
        let node = unsafe { UnsafeRef::from_raw(ptr::from_ref(node)) };

        self.waiters
            .entry(node.addr)
            .or_insert_with(|| LinkedList::new(FutexWaiterAdapter::new()))
            .push_back(node);
    }

    /// # Safety
    ///
    /// - `node` must NOT be linked to an existing waiter list.
    unsafe fn add_async_waiter(&mut self, node: Arc<FutexWaiter>) {
        // SAFETY: `node` is not linked by the guarantees of the caller.
        let node = unsafe { UnsafeRef::from_raw(Arc::into_raw(node)) };

        self.waiters
            .entry(node.addr)
            .or_insert_with(|| LinkedList::new(FutexWaiterAdapter::new()))
            .push_back(node);
    }

    /// # Safety
    ///
    /// - `node` must point to a valid instance of `FutexWaiter`.
    /// - `node` must be inside the wait list associated with `node.addr`.
    #[track_caller]
    pub(super) unsafe fn remove_waiter(&mut self, node: &FutexWaiter) {
        debug_assert!(node.link.is_linked());

        let Entry::Occupied(mut wl) = self.waiters.entry(node.addr) else {
            panic!("node was not a valid `FutexWaiter`");
        };

        // SAFETY: `node` must be inside the wait list associated with `node.addr`.
        let node = unsafe {
            let Some(node) = wl
                .get_mut()
                .cursor_mut_from_ptr(ptr::from_ref(node))
                .remove()
            else {
                panic!("node was not a valid `FutexWaiter`")
            };
            node
        };

        if node.async_data.is_some() {
            // SAFETY: all async entries must be managed by an Arc.
            unsafe {
                Arc::from_raw(UnsafeRef::into_raw(node));
            }
        }

        if wl.get().is_empty() {
            wl.remove();
        }
    }
}

/// Adds this agent to the wait queue for the address pointed to by `buffer[offset..]`.
///
/// # Safety
///
/// - `addr` must be a multiple of `std::mem::size_of::<E>()`.
/// - `buffer` must contain at least `std::mem::size_of::<E>()` bytes to read starting from `usize`.
// our implementation guarantees that `SharedArrayBuffer` is always aligned to `u64` at minimum.
pub(super) unsafe fn wait<E: Element + PartialEq>(
    buffer: &SharedArrayBuffer,
    buf_len: usize,
    offset: usize,
    check: E,
    timeout: Option<Duration>,
) -> JsResult<AtomicsWaitResult> {
    // 11. Let block be buffer.[[ArrayBufferData]].
    // 12. Let offset be typedArray.[[ByteOffset]].
    // 13. Let byteIndexInBuffer be (i × 4) + offset.
    let buffer = &buffer.bytes_with_len(buf_len)[offset..];

    // 14. Let WL be GetWaiterList(block, indexedPosition).
    // 18. Perform EnterCriticalSection(WL).
    let mut waiters = FutexWaiters::get()?;

    // 13. Let elementType be TypedArrayElementType(typedArray).
    // 14. Let w be GetValueFromBuffer(buffer, indexedPosition, elementType, true, SeqCst).

    // SAFETY: The safety of this operation is guaranteed by the caller.
    let value = unsafe { E::read(SliceRef::AtomicSlice(buffer)).load(Ordering::SeqCst) };

    // 20. If v ≠ w, then
    // a. Perform LeaveCriticalSection(WL).
    // b. If mode is sync, return "not-equal".
    if check != value {
        return Ok(AtomicsWaitResult::NotEqual);
    }

    // 23. Let now be the time value (UTC) identifying the current time.
    // 24. Let additionalTimeout be an implementation-defined non-negative mathematical value.
    // 25. Let timeoutTime be ℝ(now) + t + additionalTimeout.
    // 26. NOTE: When t is +∞, timeoutTime is also +∞.
    let timeout_time = timeout.map(|to| (Instant::now(), to));

    // 27. Let waiterRecord be a new Waiter Record { [[AgentSignifier]]: thisAgent, [[PromiseCapability]]: promiseCapability, [[TimeoutTime]]: timeoutTime, [[Result]]: "ok" }.
    // ensure we can have aliased pointers to the waiter in a sound way.
    let waiter = FutexWaiter::new_sync(buffer.as_ptr().addr());

    // 28. Perform AddWaiter(WL, waiterRecord).
    // SAFETY: waiter is valid and we call `remove_waiter` below.
    unsafe {
        waiters.add_waiter(&waiter);
    }

    // 18. Let notified be SuspendAgent(WL, W, t).

    // `SuspendAgent(WL, W, t)`
    // https://tc39.es/ecma262/#sec-suspendthisagent

    // In a couple of places here we early return without removing the waiter from
    // the waiters list. This could seem unsound, but in reality all our early
    // returns are because the mutex is poisoned, and if that's the case then
    // no other thread can read the pointer to the waiter, so we can return safely.
    let result = loop {
        if !waiter.link.is_linked() {
            break AtomicsWaitResult::Ok;
        }

        if let Some((start, timeout)) = timeout_time {
            let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                break AtomicsWaitResult::TimedOut;
            };

            // This doesn't use `wait_timeout_while` because it has to mutably borrow `waiter`,
            // which is a big nono since we have pointers to that location while the borrow is
            // active.
            waiters = waiter
                .cond_var
                .wait_timeout(waiters, remaining)
                .map_err(|_| {
                    JsNativeError::typ()
                        .with_message("failed to synchronize with the agent cluster")
                })?
                .0;
        } else {
            waiters = waiter.cond_var.wait(waiters).map_err(|_| {
                JsNativeError::typ().with_message("failed to synchronize with the agent cluster")
            })?;
        }
    };

    // 19. If notified is true, then
    //     a. Assert: W is not on the list of waiters in WL.
    // 20. Else,
    //     a. Perform RemoveWaiter(WL, W).
    if waiter.link.is_linked() {
        // SAFETY: waiter is valid and contained in its waiter list if it is still linked.
        unsafe {
            waiters.remove_waiter(&waiter);
        }
    }

    // 21. Perform LeaveCriticalSection(WL).
    drop(waiters);

    // 22. If notified is true, return "ok".
    // 23. Return "timed-out".
    Ok(result)
}

/// Adds this agent to the wait queue for the address pointed to by `buffer[offset..]`,
/// without blocking the execution thread.
///
/// # Safety
///
/// - `addr` must be a multiple of `std::mem::size_of::<E>()`.
/// - `buffer` must contain at least `std::mem::size_of::<E>()` bytes to read starting from `usize`.
// our implementation guarantees that `SharedArrayBuffer` is always aligned to `u64` at minimum.
pub(super) unsafe fn wait_async<E: Element + PartialEq>(
    buffer: &SharedArrayBuffer,
    buf_len: usize,
    offset: usize,
    check: E,
    timeout: Option<Duration>,
    functions: ResolvingFunctions,
    context: &mut Context,
) -> JsResult<AtomicsWaitResult> {
    // 11. Let block be buffer.[[ArrayBufferData]].
    // 12. Let offset be typedArray.[[ByteOffset]].
    // 13. Let byteIndexInBuffer be (i × 4) + offset.
    let buf = &buffer.bytes_with_len(buf_len)[offset..];

    // 14. Let WL be GetWaiterList(block, indexedPosition).
    // 17. Perform EnterCriticalSection(WL).
    let mut waiters = FutexWaiters::get()?;

    // 18. Let elementType be TypedArrayElementType(typedArray).
    // 19. Let w be GetValueFromBuffer(buffer, indexedPosition, elementType, true, SeqCst).
    // SAFETY: The safety of this operation is guaranteed by the caller.
    let value = unsafe { E::read(SliceRef::AtomicSlice(buf)).load(Ordering::SeqCst) };

    // 20. If v ≠ w, then
    //     a. Perform LeaveCriticalSection(WL).
    //     b. If mode is sync, return "not-equal".
    if check != value {
        return Ok(AtomicsWaitResult::NotEqual);
    }

    // 21. If t = 0 and mode is async, then
    //     a. NOTE: There is no special handling of synchronous immediate timeouts. Asynchronous immediate
    //        timeouts have special handling in order to fail fast and avoid unnecessary Promise jobs.
    //     b. Perform LeaveCriticalSection(WL).
    if let Some(timeout) = &timeout
        && timeout.is_zero()
    {
        return Ok(AtomicsWaitResult::TimedOut);
    }

    // 23. Let now be the time value (UTC) identifying the current time.
    // 24. Let additionalTimeout be an implementation-defined non-negative mathematical value.
    // 25. Let timeoutTime be ℝ(now) + t + additionalTimeout.
    // 26. NOTE: When t is +∞, timeoutTime is also +∞.

    // 27. Let waiterRecord be a new Waiter Record { [[AgentSignifier]]: thisAgent,
    //     [[PromiseCapability]]: promiseCapability, [[TimeoutTime]]: timeoutTime, [[Result]]: "ok" }.
    // ensure we can have aliased pointers to the waiter in a sound way.
    let waiter = AsyncFutexWaiter(FutexWaiter::new_async(buffer.clone(), buf.as_ptr().addr()));

    // 28. Perform AddWaiter(WL, waiterRecord).
    // SAFETY: `waiter` is pinned to the heap, so it must be valid.
    unsafe {
        waiters.add_async_waiter(Arc::clone(&waiter.0));
    }

    context.pending_waiters.insert(waiter.clone(), functions);

    // 30. Else if timeoutTime is finite, then
    //     a. Perform EnqueueAtomicsWaitAsyncTimeoutJob(WL, waiterRecord).
    if let Some(timeout) = timeout {
        // EnqueueAtomicsWaitAsyncTimeoutJob ( WL, waiterRecord )
        // https://tc39.es/ecma262/#sec-enqueueatomicswaitasynctimeoutjob
        #[derive(Debug)]
        struct WaiterDropGuard(Option<AsyncFutexWaiter>);

        impl Drop for WaiterDropGuard {
            fn drop(&mut self) {
                let Some(waiter) = self.0.take() else {
                    return;
                };
                let Ok(mut global_waiters) = FutexWaiters::get() else {
                    // Cannot cleanup a poisoned mutex.
                    return;
                };
                if waiter.0.link.is_linked() {
                    // SAFETY: the node is linked and the Arc ensures it has not been deallocated.
                    unsafe {
                        global_waiters.remove_waiter(&waiter.0);
                    }
                }
            }
        }

        // 1. Let timeoutJob be a new Job Abstract Closure with no parameters that captures WL and waiterRecord and performs the following steps when called:
        // TODO: this design is a bit suboptimal since most event loops will wait until all timeouts
        // are resolved. If we can, we should make it possible to remove the timeout if the waiter
        // was notified.
        let mut waiter_guard = WaiterDropGuard(Some(waiter));
        let job = TimeoutJob::with_realm(
            move |context| {
                //    a. Perform EnterCriticalSection(WL).
                let Some(waiter) = waiter_guard.0.take() else {
                    panic!("waiter pointer disappeared");
                };

                let mut waiters = FutexWaiters::get()?;

                //    b. If WL.[[Waiters]] contains waiterRecord, then
                if waiter.0.link.is_linked() {
                    // i. Let timeOfJobExecution be the time value (UTC) identifying the current time.
                    // ii. Assert: ℝ(timeOfJobExecution) ≥ waiterRecord.[[TimeoutTime]] (ignoring potential non-monotonicity of time values).
                    // iii. Set waiterRecord.[[Result]] to "timed-out".
                    // SAFETY: the node is linked and still valid thanks to the reference count.
                    unsafe {
                        // iv. Perform RemoveWaiter(WL, waiterRecord).
                        waiters.remove_waiter(&waiter.0);
                    }

                    let Some(functions) = context.pending_waiters.remove(&waiter) else {
                        panic!("timeout job was not executed by its original context")
                    };

                    // v. Perform NotifyWaiter(WL, waiterRecord).
                    functions
                        .resolve
                        .call(
                            &JsValue::undefined(),
                            &[js_string!("timed-out").into()],
                            context,
                        )
                        .expect("default resolving functions cannot error");
                }

                //    c. Perform LeaveCriticalSection(WL).
                //    d. Return unused.
                Ok(JsValue::undefined())
            },
            // 3. Let currentRealm be the current Realm Record.
            context.realm().clone(),
            // 2. Let now be the time value (UTC) identifying the current time.
            timeout,
        );

        // 4. Perform HostEnqueueTimeoutJob(timeoutJob, currentRealm, 𝔽(waiterRecord.[[TimeoutTime]]) - now).
        context.enqueue_job(job.into());

        // 5. Return unused.
    }

    Ok(AtomicsWaitResult::Ok)
}

/// Notifies at most `count` agents waiting on the memory address pointed to by `buffer[offset..]`.
pub(super) fn notify(
    buffer: &SharedArrayBuffer,
    offset: usize,
    count: u64,
    context: &mut Context,
) -> JsResult<u64> {
    let addr = buffer.as_ptr().addr() + offset;

    // 7. Let WL be GetWaiterList(block, indexedPosition).
    // 8. Perform EnterCriticalSection(WL).
    let mut waiters = FutexWaiters::get()?;

    // 9. Let S be RemoveWaiters(WL, c).
    // 10. For each element W of S, do
    //     a. Perform NotifyWaiter(WL, W).
    let count = waiters.notify_many(addr, count, context);

    // 11. Perform LeaveCriticalSection(WL).
    drop(waiters);

    Ok(count)
}
