#![feature(decl_macro)]
#![feature(optin_builtin_traits)]

//#![cfg_attr(test, feature(inclusive_range_syntax))]
#![no_std]
//extern crate core;

pub mod volatile;
pub mod stack_vec;
//pub mod fs;

pub use stack_vec::StackVec;

pub mod io {
}

pub mod path {
}
