#![feature(decl_macro)]
#![feature(optin_builtin_traits)]

//#![cfg_attr(test, feature(inclusive_range_syntax))]
#![no_std]

pub mod volatile;
pub mod stack_vec;

pub use stack_vec::StackVec;
