// Copyright 2012-2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use iter::Sum;
use ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign};

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;

/// A `Duration` type to represent a span of time, typically used for system
/// timeouts.
///
/// Each `Duration` is composed of a whole number of seconds and a fractional part
/// represented in nanoseconds.  If the underlying system does not support
/// nanosecond-level precision, APIs binding a system timeout will typically round up
/// the number of nanoseconds.
///
/// `Duration`s implement many common traits, including [`Add`], [`Sub`], and other
/// [`ops`] traits.
///
/// [`Add`]: ../../std/ops/trait.Add.html
/// [`Sub`]: ../../std/ops/trait.Sub.html
/// [`ops`]: ../../std/ops/index.html
///
/// # Examples
///
/// ```
/// use std::time::Duration;
///
/// let five_seconds = Duration::new(5, 0);
/// let five_seconds_and_five_nanos = five_seconds + Duration::new(0, 5);
///
/// assert_eq!(five_seconds_and_five_nanos.as_secs(), 5);
/// assert_eq!(five_seconds_and_five_nanos.subsec_nanos(), 5);
///
/// let ten_millis = Duration::from_millis(10);
/// ```
#[stable(feature = "duration", since = "1.3.0")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct Duration {
    secs: u64,
    nanos: u32, // Always 0 <= nanos < NANOS_PER_SEC
}

impl Duration {
    /// Creates a new `Duration` from the specified number of whole seconds and
    /// additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 1 billion (the number of
    /// nanoseconds in a second), then it will carry over into the seconds provided.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the carry from the nanoseconds overflows
    /// the seconds counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let five_seconds = Duration::new(5, 0);
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub fn new(secs: u64, nanos: u32) -> Duration {
        let secs = secs.checked_add((nanos / NANOS_PER_SEC) as u64)
            .expect("overflow in Duration::new");
        let nanos = nanos % NANOS_PER_SEC;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of whole seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_secs(5);
    ///
    /// assert_eq!(5, duration.as_secs());
    /// assert_eq!(0, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub fn from_secs(secs: u64) -> Duration {
        Duration { secs: secs, nanos: 0 }
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(2569);
    ///
    /// assert_eq!(2, duration.as_secs());
    /// assert_eq!(569_000_000, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub fn from_millis(millis: u64) -> Duration {
        let secs = millis / MILLIS_PER_SEC;
        let nanos = ((millis % MILLIS_PER_SEC) as u32) * NANOS_PER_MILLI;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_from_micros)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_micros(1_000_002);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(2000, duration.subsec_nanos());
    /// ```
    #[unstable(feature = "duration_from_micros", issue = "44400")]
    #[inline]
    pub fn from_micros(micros: u64) -> Duration {
        let secs = micros / MICROS_PER_SEC;
        let nanos = ((micros % MICROS_PER_SEC) as u32) * NANOS_PER_MICRO;
        Duration { secs: secs, nanos: nanos }
    }

    /// Creates a new `Duration` from the specified number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_extras)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_nanos(1_000_000_123);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(123, duration.subsec_nanos());
    /// ```
    #[unstable(feature = "duration_extras", issue = "46507")]
    #[inline]
    pub fn from_nanos(nanos: u64) -> Duration {
        let secs = nanos / (NANOS_PER_SEC as u64);
        let nanos = (nanos % (NANOS_PER_SEC as u64)) as u32;
        Duration { secs: secs, nanos: nanos }
    }

    /// Returns the number of _whole_ seconds contained by this `Duration`.
    ///
    /// The returned value does not include the fractional (nanosecond) part of the
    /// duration, which can be obtained using [`subsec_nanos`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    ///
    /// To determine the total number of seconds represented by the `Duration`,
    /// use `as_secs` in combination with [`subsec_nanos`]:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    ///
    /// assert_eq!(5.730023852,
    ///            duration.as_secs() as f64
    ///            + duration.subsec_nanos() as f64 * 1e-9);
    /// ```
    ///
    /// [`subsec_nanos`]: #method.subsec_nanos
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub fn as_secs(&self) -> u64 { self.secs }

    /// Returns the fractional part of this `Duration`, in milliseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by milliseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one thousand).
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_extras)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(5432);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_millis(), 432);
    /// ```
    #[unstable(feature = "duration_extras", issue = "46507")]
    #[inline]
    pub fn subsec_millis(&self) -> u32 { self.nanos / NANOS_PER_MILLI }

    /// Returns the fractional part of this `Duration`, in microseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by microseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one million).
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_extras, duration_from_micros)]
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_micros(1_234_567);
    /// assert_eq!(duration.as_secs(), 1);
    /// assert_eq!(duration.subsec_micros(), 234_567);
    /// ```
    #[unstable(feature = "duration_extras", issue = "46507")]
    #[inline]
    pub fn subsec_micros(&self) -> u32 { self.nanos / NANOS_PER_MICRO }

    /// Returns the fractional part of this `Duration`, in nanoseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by nanoseconds. The returned number always represents a
    /// fractional portion of a second (i.e. it is less than one billion).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(5010);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_nanos(), 10_000_000);
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    pub fn subsec_nanos(&self) -> u32 { self.nanos }

    /// Checked `Duration` addition. Computes `self + other`, returning [`None`]
    /// if overflow occurred.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 0).checked_add(Duration::new(0, 1)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(1, 0).checked_add(Duration::new(std::u64::MAX, 0)), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_add(rhs.secs) {
            let mut nanos = self.nanos + rhs.nanos;
            if nanos >= NANOS_PER_SEC {
                nanos -= NANOS_PER_SEC;
                if let Some(new_secs) = secs.checked_add(1) {
                    secs = new_secs;
                } else {
                    return None;
                }
            }
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration {
                secs,
                nanos,
            })
        } else {
            None
        }
    }

    /// Checked `Duration` subtraction. Computes `self - other`, returning [`None`]
    /// if the result would be negative or if overflow occurred.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 1).checked_sub(Duration::new(0, 0)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(0, 0).checked_sub(Duration::new(0, 1)), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_sub(rhs.secs) {
            let nanos = if self.nanos >= rhs.nanos {
                self.nanos - rhs.nanos
            } else {
                if let Some(sub_secs) = secs.checked_sub(1) {
                    secs = sub_secs;
                    self.nanos + NANOS_PER_SEC - rhs.nanos
                } else {
                    return None;
                }
            };
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs: secs, nanos: nanos })
        } else {
            None
        }
    }

    /// Checked `Duration` multiplication. Computes `self * other`, returning
    /// [`None`] if overflow occurred.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 500_000_001).checked_mul(2), Some(Duration::new(1, 2)));
    /// assert_eq!(Duration::new(std::u64::MAX - 1, 0).checked_mul(2), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        // Multiply nanoseconds as u64, because it cannot overflow that way.
        let total_nanos = self.nanos as u64 * rhs as u64;
        let extra_secs = total_nanos / (NANOS_PER_SEC as u64);
        let nanos = (total_nanos % (NANOS_PER_SEC as u64)) as u32;
        if let Some(secs) = self.secs
            .checked_mul(rhs as u64)
            .and_then(|s| s.checked_add(extra_secs)) {
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration {
                secs,
                nanos,
            })
        } else {
            None
        }
    }

    /// Checked `Duration` division. Computes `self / other`, returning [`None`]
    /// if `other == 0`.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(2, 0).checked_div(2), Some(Duration::new(1, 0)));
    /// assert_eq!(Duration::new(1, 0).checked_div(2), Some(Duration::new(0, 500_000_000)));
    /// assert_eq!(Duration::new(2, 0).checked_div(0), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[inline]
    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        if rhs != 0 {
            let secs = self.secs / (rhs as u64);
            let carry = self.secs - secs * (rhs as u64);
            let extra_nanos = carry * (NANOS_PER_SEC as u64) / (rhs as u64);
            let nanos = self.nanos / rhs + (extra_nanos as u32);
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs: secs, nanos: nanos })
        } else {
            None
        }
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs).expect("overflow when adding durations")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs).expect("overflow when subtracting durations")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs).expect("overflow when multiplying duration by scalar")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs).expect("divide by zero error when dividing duration by scalar")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

#[stable(feature = "duration_sum", since = "1.16.0")]
impl Sum for Duration {
    fn sum<I: Iterator<Item=Duration>>(iter: I) -> Duration {
        iter.fold(Duration::new(0, 0), |a, b| a + b)
    }
}

#[stable(feature = "duration_sum", since = "1.16.0")]
impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item=&'a Duration>>(iter: I) -> Duration {
        iter.fold(Duration::new(0, 0), |a, b| a + *b)
    }
}

#[cfg(test)]
mod tests {
    use super::Duration;

    #[test]
    fn creation() {
        assert!(Duration::from_secs(1) != Duration::from_secs(0));
        assert_eq!(Duration::from_secs(1) + Duration::from_secs(2),
                   Duration::from_secs(3));
        assert_eq!(Duration::from_millis(10) + Duration::from_secs(4),
                   Duration::new(4, 10 * 1_000_000));
        assert_eq!(Duration::from_millis(4000), Duration::new(4, 0));
    }

    #[test]
    fn secs() {
        assert_eq!(Duration::new(0, 0).as_secs(), 0);
        assert_eq!(Duration::from_secs(1).as_secs(), 1);
        assert_eq!(Duration::from_millis(999).as_secs(), 0);
        assert_eq!(Duration::from_millis(1001).as_secs(), 1);
    }

    #[test]
    fn nanos() {
        assert_eq!(Duration::new(0, 0).subsec_nanos(), 0);
        assert_eq!(Duration::new(0, 5).subsec_nanos(), 5);
        assert_eq!(Duration::new(0, 1_000_000_001).subsec_nanos(), 1);
        assert_eq!(Duration::from_secs(1).subsec_nanos(), 0);
        assert_eq!(Duration::from_millis(999).subsec_nanos(), 999 * 1_000_000);
        assert_eq!(Duration::from_millis(1001).subsec_nanos(), 1 * 1_000_000);
    }

    #[test]
    fn add() {
        assert_eq!(Duration::new(0, 0) + Duration::new(0, 1),
                   Duration::new(0, 1));
        assert_eq!(Duration::new(0, 500_000_000) + Duration::new(0, 500_000_001),
                   Duration::new(1, 1));
    }

    #[test]
    fn checked_add() {
        assert_eq!(Duration::new(0, 0).checked_add(Duration::new(0, 1)),
                   Some(Duration::new(0, 1)));
        assert_eq!(Duration::new(0, 500_000_000).checked_add(Duration::new(0, 500_000_001)),
                   Some(Duration::new(1, 1)));
        assert_eq!(Duration::new(1, 0).checked_add(Duration::new(::u64::MAX, 0)), None);
    }

    #[test]
    fn sub() {
        assert_eq!(Duration::new(0, 1) - Duration::new(0, 0),
                   Duration::new(0, 1));
        assert_eq!(Duration::new(0, 500_000_001) - Duration::new(0, 500_000_000),
                   Duration::new(0, 1));
        assert_eq!(Duration::new(1, 0) - Duration::new(0, 1),
                   Duration::new(0, 999_999_999));
    }

    #[test]
    fn checked_sub() {
        let zero = Duration::new(0, 0);
        let one_nano = Duration::new(0, 1);
        let one_sec = Duration::new(1, 0);
        assert_eq!(one_nano.checked_sub(zero), Some(Duration::new(0, 1)));
        assert_eq!(one_sec.checked_sub(one_nano),
                   Some(Duration::new(0, 999_999_999)));
        assert_eq!(zero.checked_sub(one_nano), None);
        assert_eq!(zero.checked_sub(one_sec), None);
    }

    #[test] #[should_panic]
    fn sub_bad1() {
        Duration::new(0, 0) - Duration::new(0, 1);
    }

    #[test] #[should_panic]
    fn sub_bad2() {
        Duration::new(0, 0) - Duration::new(1, 0);
    }

    #[test]
    fn mul() {
        assert_eq!(Duration::new(0, 1) * 2, Duration::new(0, 2));
        assert_eq!(Duration::new(1, 1) * 3, Duration::new(3, 3));
        assert_eq!(Duration::new(0, 500_000_001) * 4, Duration::new(2, 4));
        assert_eq!(Duration::new(0, 500_000_001) * 4000,
                   Duration::new(2000, 4000));
    }

    #[test]
    fn checked_mul() {
        assert_eq!(Duration::new(0, 1).checked_mul(2), Some(Duration::new(0, 2)));
        assert_eq!(Duration::new(1, 1).checked_mul(3), Some(Duration::new(3, 3)));
        assert_eq!(Duration::new(0, 500_000_001).checked_mul(4), Some(Duration::new(2, 4)));
        assert_eq!(Duration::new(0, 500_000_001).checked_mul(4000),
                   Some(Duration::new(2000, 4000)));
        assert_eq!(Duration::new(::u64::MAX - 1, 0).checked_mul(2), None);
    }

    #[test]
    fn div() {
        assert_eq!(Duration::new(0, 1) / 2, Duration::new(0, 0));
        assert_eq!(Duration::new(1, 1) / 3, Duration::new(0, 333_333_333));
        assert_eq!(Duration::new(99, 999_999_000) / 100,
                   Duration::new(0, 999_999_990));
    }

    #[test]
    fn checked_div() {
        assert_eq!(Duration::new(2, 0).checked_div(2), Some(Duration::new(1, 0)));
        assert_eq!(Duration::new(1, 0).checked_div(2), Some(Duration::new(0, 500_000_000)));
        assert_eq!(Duration::new(2, 0).checked_div(0), None);
    }
}
