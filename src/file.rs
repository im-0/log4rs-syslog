#![forbid(unsafe_code)]

use std;

use libc;
use log;
use log4rs;
use syslog;

#[derive(Deserialize)]
struct SyslogAppenderOpenlogConfig {
    ident: String,
    option: syslog::LogOption,
    facility: syslog::Facility,
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[allow(non_camel_case_types)]
enum FakeLibcLogLevel {
    LOG_EMERG,
    LOG_ALERT,
    LOG_CRIT,
    LOG_ERR,
    LOG_WARNING,
    LOG_NOTICE,
    LOG_INFO,
    LOG_DEBUG,
}

type LevelMapConf = std::collections::BTreeMap<log::Level, FakeLibcLogLevel>;

#[derive(Deserialize)]
struct SyslogAppenderConfig {
    openlog: Option<SyslogAppenderOpenlogConfig>,
    encoder: Option<log4rs::encode::EncoderConfig>,
    level_map: Option<LevelMapConf>,
}

struct SyslogAppenderDeserializer;

impl log4rs::config::Deserialize for SyslogAppenderDeserializer {
    type Trait = dyn log4rs::append::Append;
    type Config = SyslogAppenderConfig;

    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &log4rs::config::Deserializers,
    ) -> anyhow::Result<Box<Self::Trait>> {
        let mut builder = syslog::SyslogAppender::builder();

        if let Some(openlog_conf) = config.openlog {
            builder = builder.openlog(
                &openlog_conf.ident,
                openlog_conf.option,
                openlog_conf.facility,
            );
        };

        if let Some(encoder_conf) = config.encoder {
            builder = builder.encoder(deserializers.deserialize(&encoder_conf.kind, encoder_conf.config)?);
        }

        if let Some(level_map) = config.level_map {
            let mut map = std::collections::BTreeMap::new();
            for (level, libc_level) in level_map {
                let libc_level = match libc_level {
                    FakeLibcLogLevel::LOG_EMERG => libc::LOG_EMERG,
                    FakeLibcLogLevel::LOG_ALERT => libc::LOG_ALERT,
                    FakeLibcLogLevel::LOG_CRIT => libc::LOG_CRIT,
                    FakeLibcLogLevel::LOG_ERR => libc::LOG_ERR,
                    FakeLibcLogLevel::LOG_WARNING => libc::LOG_WARNING,
                    FakeLibcLogLevel::LOG_NOTICE => libc::LOG_NOTICE,
                    FakeLibcLogLevel::LOG_INFO => libc::LOG_INFO,
                    FakeLibcLogLevel::LOG_DEBUG => libc::LOG_DEBUG,
                };
                let _ = map.insert(level, libc_level);
            }

            for level in &[
                log::Level::Error,
                log::Level::Warn,
                log::Level::Info,
                log::Level::Debug,
                log::Level::Trace,
            ] {
                let _ = map.get(level)
                    .ok_or_else(|| anyhow::anyhow!("Log level missing in map: {:?}", level))?;
            }

            builder = builder.level_map(Box::new(move |l| map[&l]));
        }

        Ok(Box::new(builder.build()))
    }
}

/// Register deserializer for creating syslog appender based on log4rs configuration file.
///
/// See `./examples/from_conf.rs` for full example.
///
/// # Examples
///
/// ```
/// extern crate log4rs;
/// extern crate log4rs_syslog;
///
/// let mut deserializers = log4rs::config::Deserializers::new();
/// log4rs_syslog::register(&mut deserializers);
/// let result = log4rs::init_file("/path/to/log-conf.yaml", deserializers);
/// ```
pub fn register(deserializers: &mut log4rs::config::Deserializers) {
    deserializers.insert("libc-syslog", SyslogAppenderDeserializer);
}
