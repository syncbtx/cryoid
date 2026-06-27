
#![no_std]
#![doc = include_str!("../README.md")]
extern crate alloc;

#[allow(unused)]
type CryoID = u64;

pub mod batch;
pub mod decode;
pub mod generator;
pub mod error;
pub mod layout;
pub mod layouts;
pub mod tests;

pub use batch::BatchIterator;
pub use decode::{decode, Decoded};
pub use generator::{Gen,GenBuilder};
pub use error::CryoidError;
pub use layout::Layout;
pub use layouts::{Compact48, Snowflake63, Tagged63};