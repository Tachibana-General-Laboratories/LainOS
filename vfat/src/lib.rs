#![feature(decl_macro)]
#![feature(alloc)]
#![feature(optin_builtin_traits)]
#![feature(const_fn)]
#![feature(asm)]
#![allow(safe_packed_borrows)]

#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

#![cfg_attr(not(test), no_std)]

#[cfg(not(target_endian="little"))]
compile_error!("only little endian platforms supported");

#[cfg(test)]
extern crate core;
extern crate alloc;

extern crate hashmap_core;

pub(crate) mod hash_map {
    pub use hashmap_core::{HashMap, HashSet};
    pub use hashmap_core::map::Entry;
}

pub(crate) use alloc::boxed::Box;

#[cfg(test)] pub(crate) use std::io;
#[cfg(test)] pub(crate) use std::path;
#[cfg(test)] pub(crate) use std::ffi;
pub(crate) use alloc::borrow;

#[cfg(not(test))] pub mod io;
#[cfg(not(test))] pub(crate) mod mutex;
#[cfg(not(test))]
pub(crate) mod ffi {
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

#[cfg(not(test))]
pub mod path {
    use core::mem;

    use ffi::OsStr;
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

#[cfg(test)]
mod tests;
mod mbr;
mod util;

pub mod vfat;
pub mod traits;

pub use mbr::*;
