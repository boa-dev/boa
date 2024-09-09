// Implementation mostly based from https://github.com/v8/v8/blob/main/src/execution/futex-emulation.cc
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
#![allow(unstable_name_collisions)]

use std::{cell::UnsafeCell, sync::atomic::Ordering};

use sptr::Strict;

use crate::{
    builtins::{
        array_buffer::{utils::SliceRef, SharedArrayBuffer},
        typed_array::Element,
    },
    sys::time::{Duration, Instant},
    JsNativeError, JsResult,
};

mod sync {
    use std::sync::{Condvar, Mutex, MutexGuard};

    use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink, UnsafeRef};

    use crate::{
        small_map::{Entry, SmallMap},
        JsNativeError, JsResult,
    };

    /// A waiter of a memory address.
    #[derive(Debug, Default)]
    pub(crate) struct FutexWaiter {
        pub(super) link: LinkedListLink,
        pub(super) cond_var: Condvar,
        pub(super) waiting: bool,
        addr: usize,
    }

    intrusive_adapter!(FutexWaiterAdapter = UnsafeRef<FutexWaiter>: FutexWaiter { link: LinkedListLink });

    /// List of memory addresses and its corresponding list of waiters for that address.
    #[derive(Debug)]
    pub(super) struct FutexWaiters {
        waiters: SmallMap<usize, LinkedList<FutexWaiterAdapter>, 16>,
    }

    // SAFETY: `FutexWaiters` is not constructable outside its `get` method, and it's only exposed by
    //          a global lock, meaning the inner data of `FutexWaiters` (which includes non-Send pointers)
    //          can only be accessed by a single thread at once.
    unsafe impl Send for FutexWaiters {}

    impl FutexWaiters {
        /// Gets the map of all shared data addresses and its corresponding list of agents waiting on that location.
        pub(super) fn get() -> JsResult<MutexGuard<'static, Self>> {
            static CRITICAL_SECTION: Mutex<FutexWaiters> = Mutex::new(FutexWaiters {
                waiters: SmallMap::new(),
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
        pub(super) fn notify_many(&mut self, addr: usize, max_count: u64) -> u64 {
            let Entry::Occupied(mut wl) = self.waiters.entry(addr) else {
                return 0;
            };

            for i in 0..max_count {
                let Some(elem) = wl.get_mut().pop_front() else {
                    wl.remove();
                    return i;
                };

                elem.cond_var.notify_one();

                // SAFETY: all elements of the waiters list are guaranteed to be valid.
                unsafe {
                    (*UnsafeRef::into_raw(elem)).waiting = false;
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
        /// - `node` must always point to a valid instance of `FutexWaiter` until `node` is
        ///   removed from its linked list. This can happen by either `remove_waiter` or `notify_many`.
        pub(super) unsafe fn add_waiter(&mut self, node: *mut FutexWaiter, addr: usize) {
            // SAFETY: `node` must point to a valid instance.
            let node = unsafe {
                debug_assert!(!(*node).link.is_linked());
                (*node).waiting = true;
                (*node).addr = addr;
                UnsafeRef::from_raw(node)
            };

            self.waiters
                .entry(addr)
                .or_insert_with(|| LinkedList::new(FutexWaiterAdapter::new()))
                .push_back(node);
        }

        /// # Safety
        ///
        /// - `node` must point to a valid instance of `FutexWaiter`.
        /// - `node` must be inside the wait list associated with `node.addr`.
        pub(super) unsafe fn remove_waiter(&mut self, node: *mut FutexWaiter) {
            // SAFETY: `node` must point to a valid instance.
            let addr = unsafe { (*node).addr };

            let mut wl = match self.waiters.entry(addr) {
                Entry::Occupied(wl) => wl,
                Entry::Vacant(_) => return,
            };

            // SAFETY: `node` must be inside the wait list associated with `node.addr`.
            unsafe {
                wl.get_mut().cursor_mut_from_ptr(node).remove();
            }

            if wl.get().is_empty() {
                wl.remove();
            }
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
    // 10. Let block be buffer.[[ArrayBufferData]].
    // 11. Let WL be GetWaiterList(block, indexedPosition).
    // 12. Perform EnterCriticalSection(WL).
    let mut waiters = sync::FutexWaiters::get()?;

    let time_info = timeout.map(|timeout| (Instant::now(), timeout));

    let buffer = &buffer.bytes_with_len(buf_len)[offset..];

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
    let waiter = UnsafeCell::new(sync::FutexWaiter::default());
    let waiter_ptr = waiter.get();

    // SAFETY: waiter is valid and we call `remove_node` below.
    unsafe {
        waiters.add_waiter(waiter_ptr, buffer.as_ptr().addr());
    }

    // 18. Let notified be SuspendAgent(WL, W, t).

    // `SuspendAgent(WL, W, t)`
    // https://tc39.es/ecma262/#sec-suspendthisagent

    let result = loop {
        // SAFETY: waiter is still valid
        if unsafe { !(*waiter_ptr).waiting } {
            break AtomicsWaitResult::Ok;
        }

        if let Some((start, timeout)) = time_info {
            let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                break AtomicsWaitResult::TimedOut;
            };

            // Since the mutex is poisoned, `waiter` cannot be read from other threads, meaning
            // we can return directly.
            // This doesn't use `wait_timeout_while` because it has to mutably borrow `waiter`,
            // which is a big nono since we have pointers to that location while the borrow is
            // active.
            // SAFETY: waiter is still valid
            waiters = unsafe {
                (*waiter_ptr)
                    .cond_var
                    .wait_timeout(waiters, remaining)
                    .map_err(|_| {
                        JsNativeError::typ()
                            .with_message("failed to synchronize with the agent cluster")
                    })?
                    .0
            };
        } else {
            // SAFETY: waiter is still valid
            waiters = unsafe {
                (*waiter_ptr).cond_var.wait(waiters).map_err(|_| {
                    JsNativeError::typ()
                        .with_message("failed to synchronize with the agent cluster")
                })?
            };
        }
    };

    // SAFETY: waiter is valid and contained in its waiter list if `waiting == true`.
    unsafe {
        // 20. Else,
        //     a. Perform RemoveWaiter(WL, W).
        if (*waiter_ptr).waiting {
            waiters.remove_waiter(waiter_ptr);
        } else {
            // 19. If notified is true, then
            //     a. Assert: W is not on the list of waiters in WL.
            debug_assert!(!(*waiter_ptr).link.is_linked());
        }
    }

    // 21. Perform LeaveCriticalSection(WL).
    drop(waiters);

    // 22. If notified is true, return "ok".
    // 23. Return "timed-out".
    Ok(result)
}

/// Notifies at most `count` agents waiting on the memory address pointed to by `buffer[offset..]`.
pub(super) fn notify(buffer: &SharedArrayBuffer, offset: usize, count: u64) -> JsResult<u64> {
    let addr = buffer.as_ptr().addr() + offset;

    // 7. Let WL be GetWaiterList(block, indexedPosition).
    // 8. Perform EnterCriticalSection(WL).
    let mut waiters = sync::FutexWaiters::get()?;

    // 9. Let S be RemoveWaiters(WL, c).
    // 10. For each element W of S, do
    //     a. Perform NotifyWaiter(WL, W).
    let count = waiters.notify_many(addr, count);

    // 11. Perform LeaveCriticalSection(WL).
    drop(waiters);

    Ok(count)
}
