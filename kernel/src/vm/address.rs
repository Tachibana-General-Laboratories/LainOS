use core::fmt;
use core::ptr::NonNull;

/// A virtual address.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualAddr(usize);

/// A physical address.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalAddr(usize);

macro_rules! impl_for {
    ($tag:ident => $T:tt) => {
        impl<T: Sized> From<*mut T> for $T {
            fn from(raw_ptr: *mut T) -> $T {
                $T(raw_ptr as usize)
            }
        }

        impl $T {
            /// Returns the inner address of `self`.
            pub const fn as_ptr(&self) -> *const u8 {
                self.0 as *const u8
            }

            /// Returns the inner address of `self`.
            ///
            /// # Safety
            ///
            /// This method is marked `unsafe` because it can be used to create
            /// multiple mutable aliases to the address represented by `self`. The
            /// caller must ensure that they do not alias.
            pub const unsafe fn as_mut_ptr<T>(&mut self) -> *mut T {
                self.0 as *mut T
            }

            /// Returns the inner address of `self` as a `usize`.
            pub const fn as_usize(&self) -> usize {
                self.0
            }

            /// Returns the inner address of `self` as a `u64`.
            #[cfg(target_pointer_width = "64")]
            pub const fn as_u64(&self) -> u64 {
                self.0 as u64
            }
        }

        impl fmt::Debug for $T {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({:#016X})", stringify!($T), self.0)
            }
        }

        impl fmt::Display for $T {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({:#016X})", stringify!($tag), self.0)
            }
        }
    }
}

impl_for!(V => VirtualAddr);
impl_for!(P => PhysicalAddr);

impl<T: ?Sized> From<NonNull<T>> for VirtualAddr {
    fn from(raw: NonNull<T>) -> VirtualAddr {
        VirtualAddr(raw.as_opaque().as_ptr() as usize)
    }
}

impl From<usize> for PhysicalAddr {
    fn from(raw: usize) -> PhysicalAddr {
        PhysicalAddr(raw)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<u64> for PhysicalAddr {
    fn from(raw: u64) -> PhysicalAddr {
        PhysicalAddr(raw as usize)
    }
}
