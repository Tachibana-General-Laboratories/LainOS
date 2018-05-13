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
pub use util::{SliceExt, VecExt};
pub use mutex::Mutex;

pub mod io;

#[cfg(feature = "std_shim")]
pub use core::{
    any,
    ascii,
    //borrow,
    cell,
    char,
    clone,
    cmp,
    convert,
    default,
    f32,
    f64,
    //fmt,
    hash,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    iter,
    marker,
    mem,
    num,
    ops,
    option,
    prelude,
    ptr,
    result,
    //slice,
    //str,
    //sync,
    time,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    /*
    arch,
    array,
    heap,
    intrinsics,
    nonzero,
    panic,
    panicking,
    raw,
    simd,
    */
};

#[cfg(feature = "std_shim")]
pub use alloc::{
    arc,
    binary_heap,
    borrow,
    boxed,
    btree_map,
    btree_set,
    fmt,
    linked_list,
    rc,
    slice,
    str,
    string,
    vec,
    vec_deque,
    //allocator,
    //heap,
    //raw_vec,
};

#[cfg(feature = "std_shim")]
pub mod collections {
    pub mod hash_map {
        pub use hashmap_core::{HashMap, HashSet};
        pub use hashmap_core::map::Entry;
    }
}

#[cfg(feature = "std_shim")]
pub mod sync {
    pub use super::mutex::{Mutex, MutexGuard};
    pub use core::sync::atomic;
}

#[cfg(feature = "std_shim")]
pub mod ffi {
    use core::str::from_utf8;
    use core::mem;

    pub struct OsStr(pub [u8]);

    impl OsStr {
        pub fn to_str(&self) -> Option<&str> {
            from_utf8(&self.0).ok()
        }
    }

    impl AsRef<OsStr> for OsStr {
        fn as_ref(&self) -> &OsStr {
            self
        }
    }

    impl AsRef<OsStr> for str {
        fn as_ref(&self) -> &OsStr {
            unsafe { mem::transmute(self) }
        }
    }

    impl AsRef<OsStr> for [u8] {
        fn as_ref(&self) -> &OsStr {
            unsafe { mem::transmute(self) }
        }
    }
}

#[cfg(feature = "std_shim")]
pub mod path {
    use core::mem;
    use super::ffi::OsStr;

    pub struct Path {
        inner: OsStr,
    }

    impl Path {
        pub fn components(&self) -> impl Iterator<Item=Component> {
            self.inner.0.split(|&c| c == b'/')
                .filter(|s| s.len() > 0)
                .map(|s| s.as_ref())
                .map(Component::Normal)
                    /*
                .map(|s| if s.len() == 0 {
                    Component::RootDir
                } else {
                    Component::Normal(s.as_ref())
                })
                */
        }
    }

    impl AsRef<Path> for str {
        fn as_ref(&self) -> &Path {
            unsafe { mem::transmute(self) }
        }
    }


    /*
    pub struct Components<'a> {
        path: &'a [u8],
    }

    impl<'a> Iterator for Components<'a> {
        type Item = Component<'a>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.path.len() == 0 {
                None
            }
            let mut split = self.path.splitn(1, '/');
            let name = split.next()?;
            match split.next() {
                Some(path) => self.path = path,
                None => self.path = path,
            }

            name
        }
    }
    */

    pub enum Component<'a> {
        Normal(&'a OsStr),
        //RootDir,
    }
}
