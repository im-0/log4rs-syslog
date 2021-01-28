//! Very simple syslog appender for the log4rs based on the libc's syslog() function.
//! Supports only *nix systems.
//!
//! Source code and examples: <https://github.com/im-0/log4rs-syslog>

#![cfg_attr(feature = "unstable", warn(unreachable_pub))]
#![warn(unused_results)]

// For benchmark.
#![cfg_attr(feature = "unstable", feature(test))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate log4rs;
extern crate log;
#[cfg(feature = "config_parsing")]
extern crate serde;
#[cfg(feature = "config_parsing")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "unstable")]
extern crate test; // For benchmark.

#[cfg(target_family = "unix")]
#[cfg(feature = "config_parsing")]
mod file;
#[cfg(target_family = "unix")]
#[cfg(feature = "config_parsing")]
pub use file::*;

#[cfg(target_family = "unix")]
mod syslog;
#[cfg(target_family = "unix")]
pub use syslog::*;
