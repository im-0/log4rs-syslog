#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate log;
extern crate log4rs;

#[cfg(target_family = "unix")]
mod syslog;
#[cfg(target_family = "unix")]
pub use syslog::*;
