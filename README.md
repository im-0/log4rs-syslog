[![Build Status](https://api.travis-ci.org/im-0/log4rs-syslog.svg?branch=master)](https://travis-ci.org/im-0/log4rs-syslog)
[![crates.io](https://img.shields.io/crates/v/log4rs-syslog.svg?maxAge=3600)](https://crates.io/crates/log4rs-syslog)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache_2.0-blue.svg)
![POSIX-only build tooling](https://img.shields.io/badge/dev_platform-POSIX-lightgrey.svg)
# log4rs-syslog

`log4rs-syslog` - very simple syslog appender for the log4rs based on the libc's syslog() function. Supports only *nix
systems.

[Documentation on docs.rs](https://docs.rs/crate/log4rs-syslog)

Features:
* Logging with or without calling openlog() with identification string, logging options and facility.
* Custom mapping between rust's `log` crate log levels and syslog's log levels.

Limitations:
* When there are multiple syslog appenders, openlog() configuration of last built appender is used.
* openlog() configuration applied when log4rs_syslog::SyslogAppenderBuilder::build() called, not on
log4rs::init_config() or log4rs::Handle::set_config().

There is no proper way to fix this limitations while using libc's interface.

## Breaking changes

**2.0 → 3.0**
* Update to `log` 0.4 and `log4rs` 0.8.

**1.0 → 2.0**
* `log4rs_syslog::register_deserializer()` renamed to `log4rs_syslog::register()`.
* `kind` in deserializable configuration file changed from `syslog` to `libc-syslog`.
* Log option constants changed from `log4rs_syslog::LOG_*` to `log4rs_syslog::LogOption::LOG_*` due to changes in 
`bitflags` crate.

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
log4rs-syslog = "3.0"
```

### Initialization based on configuration file

Example configuration file:

```yaml
appenders:
  syslog:
    kind: libc-syslog
    openlog:
      ident: log4rs-syslog-example
      option: LOG_PID | LOG_NDELAY | LOG_CONS
      facility: Daemon
    encoder:
      pattern: "{M} - {m}"
root:
  level: trace
  appenders:
    - syslog
```

Example code:

```rust,no_run
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate log4rs_syslog;
extern crate tempfile;

fn main() {
    let mut deserializers = log4rs::file::Deserializers::new();
    log4rs_syslog::register(&mut deserializers);

    // Note that configuration file should have right extension, otherwise log4rs will fail to
    // recognize format.
    log4rs::init_file("test.yaml", deserializers).unwrap();

    trace!("Example trace message");
    debug!("Example debug message");
    info!("Example information message");
    warn!("Example warning message");
    error!("Example error message");

    println!("Check your logs for new messages");
}
```

### Manual initialization

Example code:

```rust,no_run
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate log4rs_syslog;

fn main() {
    // Use custom PatternEncoder to avoid duplicate timestamps in logs.
    let encoder = Box::new(log4rs::encode::pattern::PatternEncoder::new("{M} - {m}"));

    let appender = Box::new(
        log4rs_syslog::SyslogAppender::builder()
            .encoder(encoder)
            .openlog(
                "log4rs-syslog-example",
                log4rs_syslog::LogOption::LOG_PID,
                log4rs_syslog::Facility::Daemon,
            )
            .build(),
    );

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build(
            "syslog",
            appender,
        ))
        .build(log4rs::config::Root::builder().appender("syslog").build(
            log::LevelFilter::Trace,
        ))
        .unwrap();
    log4rs::init_config(config).unwrap();

    trace!("Example trace message");
    debug!("Example debug message");
    info!("Example information message");
    warn!("Example warning message");
    error!("Example error message");

    println!("Check your logs for new messages");
}
```

### Running examples

```bash
git clone --branch master https://github.com/im-0/log4rs-syslog
cd log4rs-syslog
cargo run --example manually
cargo run --example from_conf
```
