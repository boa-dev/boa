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

use std::{
    cell::Cell,
    ptr,
    sync::{Arc, atomic::Ordering},
};

use crate::{
    Context, JsNativeError, JsResult, JsValue,
    builtins::{
        array_buffer::{SharedArrayBuffer, utils::SliceRef},
        promise::PromiseCapability,
        typed_array::Element,
    },
    job::{GenericJob, TimeoutJob},
    js_string,
    sys::time::{Duration, Instant},
};

use std::sync::{Condvar, Mutex, MutexGuard};

use intrusive_collections::{LinkedList, LinkedListLink, UnsafeRef, intrusive_adapter};
use portable_atomic::AtomicBool;
use rustc_hash::FxHashMap;
use small_btree::{Entry, SmallBTreeMap};

#[derive(Debug)]
struct AsyncWaiterData {
    buffer: SharedArrayBuffer,
    notified: AtomicBool,
    refcount: Cell<u8>,
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
    fn new_sync(addr: usize) -> Self {
        Self {
            link: LinkedListLink::new(),
            cond_var: Condvar::new(),
            addr,
            async_data: None,
        }
    }

    fn new_async(buffer: SharedArrayBuffer, addr: usize) -> UnsafeRef<Self> {
        UnsafeRef::from_box(Box::new(Self {
            link: LinkedListLink::new(),
            cond_var: Condvar::new(),
            addr,
            async_data: Some(AsyncWaiterData {
                buffer,
                notified: AtomicBool::new(true),
                refcount: Cell::new(0),
            }),
        }))
    }

    fn increase_refcount(&self) {
        if let Some(data) = &self.async_data {
            data.refcount.update(|rc| rc + 1);
        }
    }

    fn decrease_refcount(&self) {
        if let Some(data) = &self.async_data {
            data.refcount.update(|rc| rc - 1);
        }
    }

    fn refcount(&self) -> u8 {
        self.async_data
            .as_ref()
            .map_or(2, |data| data.refcount.get())
    }
}

#[derive(Debug, Clone)]
struct FutexWaiterKey(UnsafeRef<FutexWaiter>);

impl std::hash::Hash for FutexWaiterKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        UnsafeRef::into_raw(self.0.clone()).addr().hash(state);
    }
}

impl PartialEq for FutexWaiterKey {
    fn eq(&self, other: &Self) -> bool {
        UnsafeRef::into_raw(self.0.clone()).addr() == UnsafeRef::into_raw(other.0.clone()).addr()
    }
}

impl Eq for FutexWaiterKey {}

#[derive(Debug, Default)]
pub(crate) struct AsyncPendingWaiters {
    waiters: FxHashMap<FutexWaiterKey, PromiseCapability>,
}

impl AsyncPendingWaiters {
    pub(crate) fn new() -> Self {
        Self {
            waiters: FxHashMap::default(),
        }
    }

    fn insert(&mut self, waiter: UnsafeRef<FutexWaiter>, cap: PromiseCapability) {
        // Should have been added to the waiters list at this point.
        debug_assert!(waiter.link.is_linked());
        waiter.increase_refcount();
        self.waiters.insert(FutexWaiterKey(waiter), cap);
    }

    fn remove(&mut self, waiter: &UnsafeRef<FutexWaiter>) -> Option<PromiseCapability> {
        let Some(cap) = self.waiters.remove(&FutexWaiterKey(waiter.clone())) else {
            return None;
        };
        waiter.decrease_refcount();
        Some(cap)
    }

    pub(crate) fn enqueue_waiter_jobs(&mut self, context: &mut Context) -> bool {
        for (waiter, cap) in self.waiters.extract_if(|k, _| {
            k.0.async_data
                .as_ref()
                .map_or(false, |data| data.notified.load(Ordering::Relaxed))
        }) {
            // Should have been removed from the waiters list at this point.
            debug_assert!(!waiter.0.link.is_linked());
            let realm = cap
                .functions
                .resolve
                .get_function_realm(context)
                .expect("cannot fail for the default resolving functions");
            context.enqueue_job(
                GenericJob::new(
                    move |context| {
                        cap.functions.resolve.call(
                            &JsValue::undefined(),
                            &[js_string!("ok").into()],
                            context,
                        )
                    },
                    realm,
                )
                .into(),
            );

            if waiter.0.refcount() < 2 {
                // SAFETY: This is the last reference to the waiter, so it is safe to deallocate.
                unsafe { UnsafeRef::into_box(waiter.0) };
            }
        }

        !self.waiters.is_empty()
    }
}

impl Drop for AsyncPendingWaiters {
    fn drop(&mut self) {
        if self.waiters.is_empty() {
            return;
        }

        let Ok(mut global_waiters) = FutexWaiters::get() else {
            // Cannot cleanup a poisoned mutex.
            return;
        };

        for waiter in self.waiters.keys() {
            // SAFETY: All waiters in the pending waiters map must be valid.
            unsafe {
                global_waiters.remove_waiter(&waiter.0);
            }

            if waiter.0.refcount() < 2 {
                // SAFETY: This is the last reference to the waiter, so it is safe to deallocate.
                unsafe { UnsafeRef::into_box(waiter.0.clone()) };
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

                if let Some(cap) = context.pending_waiters.remove(&elem) {
                    // The waiter was part of this context, so we can safely resolve the promise from here.
                    cap.functions
                        .resolve
                        .call(&JsValue::undefined(), &[js_string!("ok").into()], context)
                        .expect("cannot fail per the spec");
                }

                if async_data.refcount.get() < 2 {
                    // SAFETY: This is the last reference to the waiter, so it is safe to deallocate.
                    unsafe { UnsafeRef::into_box(elem) };
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

        node.increase_refcount();

        self.waiters
            .entry(node.addr)
            .or_insert_with(|| LinkedList::new(FutexWaiterAdapter::new()))
            .push_back(node);
    }

    /// # Safety
    ///
    /// - `node` must point to a valid instance of `FutexWaiter`.
    /// - `node` must be inside the wait list associated with `node.addr`.
    pub(super) unsafe fn remove_waiter(&mut self, node: &FutexWaiter) {
        debug_assert!(node.link.is_linked());

        let Entry::Occupied(mut wl) = self.waiters.entry(node.addr) else {
            panic!("node was not a valid `FutexWaiter`");
        };

        // SAFETY: `node` must be inside the wait list associated with `node.addr`.
        unsafe {
            wl.get_mut()
                .cursor_mut_from_ptr(ptr::from_ref(node))
                .remove();
        }

        if let Some(data) = &node.async_data {
            data.refcount.update(|rc| rc - 1);
        }

        if wl.get().is_empty() {
            wl.remove();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AtomicsWaitResult {
    NotEqual,
    TimedOut,
    Ok,
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
    let time_info = timeout.map(|timeout| (Instant::now(), timeout));

    // 10. Let block be buffer.[[ArrayBufferData]].
    let buffer = &buffer.bytes_with_len(buf_len)[offset..];

    // 11. Let WL be GetWaiterList(block, indexedPosition).
    // 12. Perform EnterCriticalSection(WL).
    let mut waiters = FutexWaiters::get()?;

    // 13. Let elementType be TypedArrayElementType(typedArray).
    // 14. Let w be GetValueFromBuffer(buffer, indexedPosition, elementType, true, SeqCst).

    // SAFETY: The safety of this operation is guaranteed by the caller.
    let value = unsafe { E::read(SliceRef::AtomicSlice(buffer)).load(Ordering::SeqCst) };

    // 15. If v ≠ w, then
    //     a. Perform LeaveCriticalSection(WL).
    //     b. Return "not-equal".
    if check != value {
        return Ok(AtomicsWaitResult::NotEqual);
    }

    // 16. Let W be AgentSignifier().
    // 17. Perform AddWaiter(WL, W).

    // ensure we can have aliased pointers to the waiter in a sound way.
    let waiter = FutexWaiter::new_sync(buffer.as_ptr().addr());

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

        if let Some((start, timeout)) = time_info {
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
                .0
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

pub(super) unsafe fn wait_async<E: Element + PartialEq>(
    shared: &SharedArrayBuffer,
    buf_len: usize,
    offset: usize,
    check: E,
    timeout: Option<Duration>,
    capability: PromiseCapability,
    context: &mut Context,
) -> JsResult<AtomicsWaitResult> {
    // 11. Let block be buffer.[[ArrayBufferData]].
    // 12. Let offset be typedArray.[[ByteOffset]].
    // 13. Let byteIndexInBuffer be (i × 4) + offset..
    let buffer = &shared.bytes_with_len(buf_len)[offset..];

    // 14. Let WL be GetWaiterList(block, indexedPosition).
    // 17. Perform EnterCriticalSection(WL).
    let mut waiters = FutexWaiters::get()?;

    // 18. Let elementType be TypedArrayElementType(typedArray).
    // 19. Let w be GetValueFromBuffer(buffer, indexedPosition, elementType, true, SeqCst).
    // SAFETY: The safety of this operation is guaranteed by the caller.
    let value = unsafe { E::read(SliceRef::AtomicSlice(buffer)).load(Ordering::SeqCst) };

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
    // 27. Let waiterRecord be a new Waiter Record { [[AgentSignifier]]: thisAgent,
    //     [[PromiseCapability]]: promiseCapability, [[TimeoutTime]]: timeoutTime, [[Result]]: "ok" }.
    // ensure we can have aliased pointers to the waiter in a sound way.
    let waiter = FutexWaiter::new_async(shared.clone(), buffer.as_ptr().addr());

    // 23. Let now be the time value (UTC) identifying the current time.
    // 24. Let additionalTimeout be an implementation-defined non-negative mathematical value.
    // 25. Let timeoutTime be ℝ(now) + t + additionalTimeout.
    // 26. NOTE: When t is +∞, timeoutTime is also +∞.
    let timeout_time = timeout.map(|to| Instant::now() + to);

    // 28. Perform AddWaiter(WL, waiterRecord).
    // SAFETY: `waiter` is pinned to the heap, so it must be valid.
    unsafe {
        waiters.add_waiter(&*waiter);
    }

    context
        .pending_waiters
        .insert(waiter.clone(), capability.clone());

    // 30. Else if timeoutTime is finite, then
    //     a. Perform EnqueueAtomicsWaitAsyncTimeoutJob(WL, waiterRecord).
    if let Some(timeout_time) = timeout_time {
        // EnqueueAtomicsWaitAsyncTimeoutJob ( WL, waiterRecord )
        // https://tc39.es/ecma262/#sec-enqueueatomicswaitasynctimeoutjob
        #[derive(Debug)]
        struct WaiterDropGuard(UnsafeRef<FutexWaiter>);

        impl Drop for WaiterDropGuard {
            fn drop(&mut self) {
                let Ok(mut global_waiters) = FutexWaiters::get() else {
                    // Cannot cleanup a poisoned mutex.
                    return;
                };
                if self.0.link.is_linked() {
                    // The node is linked and still valid thanks to the reference count.
                    unsafe {
                        global_waiters.remove_waiter(&*self.0);
                    }
                }
                if waiter.refcount() < 2 {
                    // SAFETY: This is the last reference to the waiter, so it is safe to deallocate.
                    unsafe { UnsafeRef::into_box(waiter) }
                }
            }
        }

        let realm = context.realm().clone();
        let now = Instant::now();
        waiter.increase_refcount();
        let waiter_guard = WaiterDropGuard(waiter);
        let job = TimeoutJob::with_realm(
            move |context| {
                let waiter = waiter_guard.0.clone();
                std::mem::forget(waiter_guard);

                let waiters = FutexWaiters::get()?;

                if waiter.link.is_linked() {
                    // The node is linked and still valid thanks to the reference count.
                    unsafe {
                        waiters.remove_waiter(&waiter);
                    }
                    context.pending_waiters.remove(&waiter).expect("cannot ")
                }

                if waiter.refcount() < 2 {
                    // SAFETY: This is the last reference to the waiter, so it is safe to deallocate.
                    unsafe { UnsafeRef::into_box(waiter) }
                }
                Ok(JsValue::undefined())
            },
            context.realm().clone(),
            timeout_time - now,
        );
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
