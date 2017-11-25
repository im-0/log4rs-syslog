extern crate libc;
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
            // Custom rust log level <=> libc log level mapping.
            .level_map(Box::new(|l| match l {
                // WARNING: On linux this will broadcast error message on all consoles.
                log::LogLevel::Error => libc::LOG_EMERG,

                log::LogLevel::Warn => libc::LOG_WARNING,
                log::LogLevel::Info => libc::LOG_INFO,
                log::LogLevel::Debug | log::LogLevel::Trace => libc::LOG_DEBUG,
            }))
            .build(),
    );

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build(
            "syslog",
            appender,
        ))
        .build(log4rs::config::Root::builder().appender("syslog").build(
            log::LogLevelFilter::Trace,
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
