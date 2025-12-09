//! Clock related types and functions.

/// A monotonic instant in time, in the Boa engine.
///
/// This type is guaranteed to be monotonic, i.e. if two instants
/// are compared, the later one will always be greater than the
/// earlier one. It is also always guaranteed to be greater than
/// or equal to the Unix epoch.
///
/// This should not be used to keep dates or times, but only to
/// measure the current time in the engine.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct JsInstant {
    /// The duration of time since the Unix epoch.
    inner: std::time::Duration,
}

impl JsInstant {
    /// Creates a new `JsInstant` from the given number of seconds and nanoseconds.
    #[must_use]
    pub fn new(secs: u64, nanos: u32) -> Self {
        let inner = std::time::Duration::new(secs, nanos);
        Self::new_unchecked(inner)
    }

    /// Creates a new `JsInstant` from an unchecked duration since the Unix epoch.
    #[must_use]
    fn new_unchecked(inner: std::time::Duration) -> Self {
        Self { inner }
    }

    /// Returns the number of milliseconds since the Unix epoch.
    #[must_use]
    pub fn millis_since_epoch(&self) -> u64 {
        self.inner.as_millis() as u64
    }

    /// Returns the number of nanoseconds since the Unix epoch.
    #[must_use]
    pub fn nanos_since_epoch(&self) -> u128 {
        self.inner.as_nanos()
    }
}

/// A duration of time, inside the Boa engine.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct JsDuration {
    inner: std::time::Duration,
}

impl JsDuration {
    /// Creates a new `JsDuration` from the given number of milliseconds.
    #[must_use]
    pub fn from_millis(millis: u64) -> Self {
        Self {
            inner: std::time::Duration::from_millis(millis),
        }
    }

    /// Returns the number of milliseconds in this duration.
    #[must_use]
    pub fn as_millis(&self) -> u64 {
        self.inner.as_millis() as u64
    }

    /// Returns the number of seconds in this duration.
    #[must_use]
    pub fn as_secs(&self) -> u64 {
        self.inner.as_secs()
    }

    /// Returns the number of nanoseconds in this duration.
    #[must_use]
    pub fn as_nanos(&self) -> u128 {
        self.inner.as_nanos()
    }
}

impl From<std::time::Duration> for JsDuration {
    fn from(duration: std::time::Duration) -> Self {
        Self { inner: duration }
    }
}

impl From<JsDuration> for std::time::Duration {
    fn from(duration: JsDuration) -> Self {
        duration.inner
    }
}

macro_rules! impl_duration_ops {
    ($($trait:ident $trait_fn:ident),*) => {
        $(
            impl std::ops::$trait for JsDuration {
                type Output = JsDuration;

                #[inline]
                fn $trait_fn(self, rhs: JsDuration) -> Self::Output {
                    Self {
                        inner: std::ops::$trait::$trait_fn(self.inner, rhs.inner)
                    }
                }
            }
            impl std::ops::$trait<JsDuration> for JsInstant {
                type Output = JsInstant;

                #[inline]
                fn $trait_fn(self, rhs: JsDuration) -> Self::Output {
                    Self {
                        inner: std::ops::$trait::$trait_fn(self.inner, rhs.inner)
                    }
                }
            }
        )*
    };
}

impl_duration_ops!(Add add, Sub sub);

impl std::ops::Sub for JsInstant {
    type Output = JsDuration;

    #[inline]
    fn sub(self, rhs: JsInstant) -> Self::Output {
        JsDuration {
            inner: self
                .inner
                .checked_sub(rhs.inner)
                .expect("overflow when subtracting durations"),
        }
    }
}

/// Implement a clock that can be used to measure time.
pub trait Clock {
    /// Returns the current time.
    fn now(&self) -> JsInstant;
}

/// A clock that uses the standard system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdClock;

impl Clock for StdClock {
    fn now(&self) -> JsInstant {
        let now = std::time::SystemTime::now();
        let duration = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System clock is before Unix epoch");

        JsInstant::new_unchecked(duration)
    }
}

/// A clock that uses a fixed time, useful for testing. The internal time is in milliseconds.
///
/// This clock will always return the same time, unless it is moved forward manually. It cannot
/// be moved backward or set to a specific time.
#[derive(Debug, Clone, Default)]
pub struct FixedClock(std::cell::RefCell<u64>);

impl FixedClock {
    /// Creates a new `FixedClock` from the given number of milliseconds since the Unix epoch.
    #[must_use]
    pub fn from_millis(millis: u64) -> Self {
        Self(std::cell::RefCell::new(millis))
    }

    /// Move the clock forward by the given number of milliseconds.
    pub fn forward(&self, millis: u64) {
        *self.0.borrow_mut() += millis;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> JsInstant {
        let millis = *self.0.borrow();
        JsInstant::new_unchecked(std::time::Duration::new(
            millis / 1000,
            ((millis % 1000) * 1_000_000) as u32,
        ))
    }
}

#[test]
fn basic() {
    let now = StdClock.now();
    assert!(now.millis_since_epoch() > 0);
    assert!(now.nanos_since_epoch() > 0);

    let duration = JsDuration::from_millis(1000);
    let later = now + duration;
    assert!(later > now);

    let earlier = now - duration;
    assert!(earlier < now);

    let diff = later - earlier;
    assert_eq!(diff.as_millis(), 2000);

    let fixed = FixedClock::from_millis(0);
    let now2 = fixed.now();
    assert_eq!(now2.millis_since_epoch(), 0);
    assert!(now2 < now);

    fixed.forward(1000);
    let now3 = fixed.now();
    assert_eq!(now3.millis_since_epoch(), 1000);
    assert!(now3 > now2);

    // End of time.
    fixed.forward(u64::MAX - 1000);
    let now4 = fixed.now();
    assert_eq!(now4.millis_since_epoch(), u64::MAX);
    assert!(now4 > now3);
}
