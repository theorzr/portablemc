//! PortableMC CLI.
//! 
//! # Unstable API
//! 
//! This library is **not a stable API** and is not covered by semantic versioning: any
//! item may change or be removed in any release, including patch releases. It is only 
//! exposed so that the CLI can be embedded in other first-party distributions of 
//! PortableMC, such as the Python package. If you want to build a launcher, use the 
//! [`portablemc`](https://docs.rs/portablemc) crate instead.

#![deny(unsafe_code)]

#[cfg(feature = "__unstable")]
pub mod parse;
#[cfg(feature = "__unstable")]
pub mod format;
#[cfg(feature = "__unstable")]
pub mod output;
#[cfg(feature = "__unstable")]
pub mod cmd;
