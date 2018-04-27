#![feature(decl_macro, conservative_impl_trait)]
#![feature(entry_and_modify)]
#![feature(option_filter)]
#![allow(safe_packed_borrows)]
#![feature(pointer_methods)]

#[cfg(not(target_endian="little"))]
compile_error!("only little endian platforms supported");

extern crate fnv;
extern crate sys;

#[cfg(test)]
extern crate rand;

#[cfg(test)]
mod tests;

mod mbr;
mod util;

pub mod vfat;

pub use mbr::*;
