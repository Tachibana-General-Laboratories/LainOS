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

//#[cfg(test)]
//extern crate core;
//extern crate alloc;

#[cfg(not(test))]
extern crate sys as std;

extern crate hashmap_core;

#[cfg(test)]
mod tests;
mod mbr;
mod util;

pub mod vfat;
pub mod traits;

pub use mbr::*;
