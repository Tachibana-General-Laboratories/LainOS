#![feature(decl_macro)]
#![feature(optin_builtin_traits)]
#![feature(alloc)]
#![feature(const_fn)]
#![feature(toowned_clone_into)]

//#![cfg_attr(test, feature(inclusive_range_syntax))]
#![cfg_attr(not(test), no_std)]
#[cfg(test)]extern crate core;

extern crate alloc;
pub extern crate hashmap_core;
//pub extern crate core_io as io;

pub mod volatile;

mod stack_vec;
mod mutex;
mod util;

pub use stack_vec::StackVec;
pub use mutex::Mutex;

pub mod hash_map {
    pub use hashmap_core::{HashMap, HashSet};
    pub use hashmap_core::map::Entry;
}

pub use hash_map::{HashMap, HashSet};
pub use alloc::boxed::Box;
pub use alloc::*;

pub use util::*;

pub mod ffi {
    pub use alloc::*;
    pub struct OsStr {
        inner: [u8],
    }
    impl OsStr {
        pub fn to_str(&self) -> Option<&str> {
            ::core::str::from_utf8(&self.inner).ok()
        }
    }
    #[derive(Clone)]
    pub struct OsString {
        inner: String,
    }
}

pub use ffi::*;

pub mod prelude {
    /*
    std::marker::{Copy, Send, Sized, Sync}. The marker traits indicate fundamental properties of types.
    std::ops::{Drop, Fn, FnMut, FnOnce}. Various operations for both destructors and overloading ().
    std::mem::drop, a convenience function for explicitly dropping a value.
    */
    pub use alloc::boxed::Box;
    pub use alloc::borrow::ToOwned;
    /*
    std::clone::Clone, the ubiquitous trait that defines clone, the method for producing a copy of a value.
    std::cmp::{PartialEq, PartialOrd, Eq, Ord }. The comparison traits, which implement the comparison operators and are often seen in trait bounds.
    std::convert::{AsRef, AsMut, Into, From}. Generic conversions, used by savvy API authors to create overloaded methods.
    std::default::Default, types that have default values.
    std::iter::{Iterator, Extend, IntoIterator, DoubleEndedIterator, ExactSizeIterator}. Iterators of various kinds.
    std::option::Option::{self, Some, None}. A type which expresses the presence or absence of a value. This type is so commonly used, its variants are also exported.
    std::result::Result::{self, Ok, Err}. A type for functions that may succeed or fail. Like Option, its variants are exported as well.
    std::slice::SliceConcatExt, a trait that exists for technical reasons, but shouldn't have to exist. It provides a few useful methods on slices.
    */

    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
}
