#![feature(decl_macro, conservative_impl_trait)]
#![feature(entry_and_modify)]
#![allow(safe_packed_borrows)]

#[cfg(not(target_endian="little"))]
compile_error!("only little endian platforms supported");

#[cfg(test)]
mod tests;
mod mbr;
mod traits;
mod util;

pub mod vfat;

pub use mbr::*;
pub use traits::*;
