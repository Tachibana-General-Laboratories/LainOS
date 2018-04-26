#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(pointer_methods)]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;
extern crate sys;

pub mod timer;
pub mod uart;
pub mod gpio;
pub mod common;
pub mod atags;
pub mod interrupt;
pub mod mbox;
