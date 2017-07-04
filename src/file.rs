use std;

use log4rs;
use syslog;

#[derive(Deserialize)]
struct SyslogAppenderOpenlogConfig {
    ident: String,
    option: syslog::LogOption,
    facility: syslog::Facility,
}

#[derive(Deserialize)]
struct SyslogAppenderConfig {
    openlog: Option<SyslogAppenderOpenlogConfig>,
    encoder: Option<log4rs::encode::EncoderConfig>,
}

struct SyslogAppenderDeserializer;

impl log4rs::file::Deserialize for SyslogAppenderDeserializer {
    type Trait = log4rs::append::Append;
    type Config = SyslogAppenderConfig;

    fn deserialize(
        &self,
        config: Self::Config,
        deserializers: &log4rs::file::Deserializers,
    ) -> Result<Box<Self::Trait>, Box<std::error::Error + Sync + Send>> {
        let mut builder = syslog::SyslogAppender::builder();

        if let Some(openlog_conf) = config.openlog {
            builder = builder.openlog(
                &openlog_conf.ident,
                openlog_conf.option,
                openlog_conf.facility,
            );
        };

        if let Some(encoder_conf) = config.encoder {
            builder = builder.encoder(deserializers.deserialize(
                &encoder_conf.kind,
                encoder_conf.config,
            )?);
        }

        Ok(Box::new(builder.build()))
    }
}

#[cfg(feature = "file")]
pub fn register_deserializer(deserializers: &mut log4rs::file::Deserializers) {
    deserializers.insert("syslog", SyslogAppenderDeserializer);
}
