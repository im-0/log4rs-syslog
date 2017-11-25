#[macro_use]
extern crate log;
extern crate log4rs;
extern crate log4rs_syslog;
extern crate tempfile;

fn main() {
    use std::io::Write;

    let mut deserializers = log4rs::file::Deserializers::new();
    log4rs_syslog::register(&mut deserializers);

    let yaml_conf = br#"
appenders:
  syslog:
    kind: libc-syslog
    openlog:
      ident: log4rs-syslog-example
      option: LOG_PID | LOG_NDELAY | LOG_CONS
      facility: Daemon
    level_map:
      # WARNING: On linux this will broadcast error message on all consoles.
      Error: LOG_EMERG

      Warn: LOG_WARNING
      Info: LOG_INFO
      Debug: LOG_DEBUG
      Trace: LOG_DEBUG
    encoder:
      pattern: "{M} - {m}"
root:
  level: trace
  appenders:
    - syslog
"#;
    // Note that configuration file should have right extension, otherwise log4rs will fail to recognize format.
    let mut tmp_conf = tempfile::NamedTempFileOptions::new()
        .suffix(".yaml")
        .create()
        .unwrap();
    tmp_conf.write_all(yaml_conf).unwrap();
    tmp_conf.flush().unwrap();

    log4rs::init_file(tmp_conf.path(), deserializers).unwrap();

    trace!("Example trace message");
    debug!("Example debug message");
    info!("Example information message");
    warn!("Example warning message");
    error!("Example error message");

    println!("Check your logs for new messages");
}
