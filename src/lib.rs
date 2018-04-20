//! Very simple syslog appender for the log4rs based on the libc's syslog() function.
//! Supports only *nix systems.
//!
//! Source code and examples: <https://github.com/im-0/log4rs-syslog>

#![cfg_attr(feature = "unstable", warn(unreachable_pub))]
#![warn(unused_results)]
#![cfg_attr(feature = "cargo-clippy", warn(empty_line_after_outer_attr))]
#![cfg_attr(feature = "cargo-clippy", warn(filter_map))]
#![cfg_attr(feature = "cargo-clippy", warn(if_not_else))]
#![cfg_attr(feature = "cargo-clippy", warn(mut_mut))]
#![cfg_attr(feature = "cargo-clippy", warn(non_ascii_literal))]
#![cfg_attr(feature = "cargo-clippy", warn(option_map_unwrap_or))]
#![cfg_attr(feature = "cargo-clippy", warn(option_map_unwrap_or_else))]
#![cfg_attr(feature = "cargo-clippy", warn(single_match_else))]
#![cfg_attr(feature = "cargo-clippy", warn(wrong_pub_self_convention))]
#![cfg_attr(feature = "cargo-clippy", warn(use_self))]
#![cfg_attr(feature = "cargo-clippy", warn(used_underscore_binding))]
#![cfg_attr(feature = "cargo-clippy", warn(print_stdout))]
#![cfg_attr(feature = "cargo-clippy", warn(else_if_without_else))]

// For benchmark.
#![cfg_attr(feature = "unstable", feature(test))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate log4rs;
extern crate log;
#[cfg(feature = "file")]
extern crate serde;
#[cfg(feature = "file")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "unstable")]
extern crate test; // For benchmark.

#[cfg(target_family = "unix")]
#[cfg(feature = "file")]
mod file;
#[cfg(target_family = "unix")]
#[cfg(feature = "file")]
pub use file::*;

#[cfg(target_family = "unix")]
mod syslog;
#[cfg(target_family = "unix")]
pub use syslog::*;
