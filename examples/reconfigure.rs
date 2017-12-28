extern crate libc;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate log4rs_syslog;

trait MaybeOpenLog {
    fn maybe_openlog(self, ident: Option<&str>) -> Self;
}

impl MaybeOpenLog for log4rs_syslog::SyslogAppenderBuilder {
    fn maybe_openlog(self, ident: Option<&str>) -> Self {
        match ident {
            Some(ident) => {
                self.openlog(
                    ident,
                    log4rs_syslog::LogOption::empty(),
                    log4rs_syslog::Facility::Daemon,
                )
            },
            None => self,
        }
    }
}

fn get_conf(ident: Option<&str>) -> log4rs::config::Config {
    let appender = Box::new(
        log4rs_syslog::SyslogAppender::builder()
            .maybe_openlog(ident)
            .build(),
    );

    log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build(
            "syslog",
            appender,
        ))
        .build(log4rs::config::Root::builder().appender("syslog").build(
            log::LevelFilter::Trace,
        ))
        .unwrap()
}

fn main() {
    let handle = log4rs::init_config(get_conf(None)).unwrap();
    warn!("log4rs-syslog test message 1, NO IDENT");

    handle.set_config(get_conf(Some("ident1")));
    warn!("log4rs-syslog test message 2, IDENT == \"ident1\"");

    handle.set_config(get_conf(Some("ident2")));
    warn!("log4rs-syslog test message 3, IDENT == \"ident2\"");

    handle.set_config(get_conf(None));
    warn!("log4rs-syslog test message 4, NO IDENT");

    println!("Check your logs for test messages");
}
