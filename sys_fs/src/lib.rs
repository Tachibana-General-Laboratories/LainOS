#![feature(decl_macro, conservative_impl_trait)]
#![feature(entry_and_modify)]
#![feature(option_filter)]
#![feature(pointer_methods)]
#![feature(alloc)]

#![allow(safe_packed_borrows)]

#![no_std]

#[cfg(not(target_endian="little"))]
compile_error!("only little endian platforms supported");

//extern crate alloc;

//extern crate fnv;
extern crate sys;

#[cfg(test)]
extern crate rand;

#[cfg(test)]
mod tests;

mod mbr;

pub mod vfat;

pub use mbr::*;
