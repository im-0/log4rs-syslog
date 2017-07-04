use std;

use libc;
use log;
use log4rs;
#[cfg(feature = "file")]
use serde;

const DEFAULT_BUF_SIZE: usize = 4096;

struct BufWriter {
    buf: std::io::Cursor<Vec<u8>>,
}

impl BufWriter {
    pub fn new() -> Self {
        Self {
            buf: std::io::Cursor::new(Vec::with_capacity(DEFAULT_BUF_SIZE)),
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf.into_inner()
    }
}

impl std::io::Write for BufWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.flush()
    }
}

impl log4rs::encode::Write for BufWriter {}

pub type LevelMap = Fn(log::LogLevel) -> libc::c_int + Send + Sync;

pub struct SyslogAppender {
    encoder: Box<log4rs::encode::Encode>,
    ident: Option<String>,
    level_map: Option<Box<LevelMap>>,
}

impl std::fmt::Debug for SyslogAppender {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "SyslogAppender {{encoder: {:?}, ident: {:?}, level_map: {}}}",
            self.encoder,
            self.ident,
            match self.level_map {
                Some(_) => "Some(_)",
                None => "None",
            }
        )
    }
}

impl SyslogAppender {
    pub fn builder() -> SyslogAppenderBuilder {
        SyslogAppenderBuilder {
            encoder: None,
            ident: None,
            level_map: None,
        }
    }
}

impl Drop for SyslogAppender {
    fn drop(&mut self) {
        if self.ident.is_some() {
            unsafe {
                libc::closelog();
            }
        }
    }
}

impl log4rs::append::Append for SyslogAppender {
    fn append(&self, record: &log::LogRecord) -> std::result::Result<(), Box<std::error::Error + Sync + Send>> {
        let mut buf = BufWriter::new();

        self.encoder.encode(&mut buf, record)?;
        let mut buf = buf.into_inner();
        buf.push(0);

        let level = match self.level_map {
            Some(ref level_map) => level_map(record.level()),

            None => {
                match record.level() {
                    log::LogLevel::Error => libc::LOG_ERR,
                    log::LogLevel::Warn => libc::LOG_WARNING,
                    log::LogLevel::Info => libc::LOG_INFO,
                    log::LogLevel::Debug => libc::LOG_DEBUG,
                    log::LogLevel::Trace => libc::LOG_DEBUG,
                }
            },
        };

        unsafe {
            libc::syslog(
                level,
                b"%s\0".as_ptr() as *const libc::c_char,
                buf.as_ptr() as *const libc::c_char,
            );
        }

        Ok(())
    }
}

bitflags! {
    pub struct LogOption: libc::c_int {
        const LOG_CONS   = libc::LOG_CONS;
        const LOG_NDELAY = libc::LOG_NDELAY;
        const LOG_NOWAIT = libc::LOG_NOWAIT;
        const LOG_ODELAY = libc::LOG_ODELAY;
        const LOG_PERROR = libc::LOG_PERROR;
        const LOG_PID    = libc::LOG_PID;
    }
}

#[cfg(feature = "file")]
struct LogOptionVisitor;

#[cfg(feature = "file")]
impl<'de> serde::de::Visitor<'de> for LogOptionVisitor {
    type Value = LogOption;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("list of flags separated by \"|\"")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut flags = LogOption::empty();

        let value = value.trim();
        if !value.is_empty() {
            for str_flag in value.split('|') {
                let str_flag = str_flag.trim();
                match str_flag {
                    "LOG_CONS" => flags = flags | LOG_CONS,
                    "LOG_NDELAY" => flags = flags | LOG_NDELAY,
                    "LOG_NOWAIT" => flags = flags | LOG_NOWAIT,
                    "LOG_ODELAY" => flags = flags | LOG_ODELAY,
                    "LOG_PERROR" => flags = flags | LOG_PERROR,
                    "LOG_PID" => flags = flags | LOG_PID,
                    unknown => return Err(E::custom(format!("Unknown syslog flag: \"{}\"", unknown))),
                }
            }
        }

        Ok(flags)
    }
}

#[cfg(feature = "file")]
impl<'de> serde::de::Deserialize<'de> for LogOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(LogOptionVisitor)
    }
}

#[cfg_attr(feature = "file", derive(Deserialize))]
pub enum Facility {
    Auth,
    AuthPriv,
    Cron,
    Daemon,
    Ftp,
    Kern,
    Local0,
    Local1,
    Local2,
    Local3,
    Local4,
    Local5,
    Local6,
    Local7,
    Lpr,
    Mail,
    News,
    Syslog,
    User,
    Uucp,
}

impl Into<libc::c_int> for Facility {
    fn into(self) -> libc::c_int {
        match self {
            Facility::Auth => libc::LOG_AUTH,
            Facility::AuthPriv => libc::LOG_AUTHPRIV,
            Facility::Cron => libc::LOG_CRON,
            Facility::Daemon => libc::LOG_DAEMON,
            Facility::Ftp => libc::LOG_FTP,
            Facility::Kern => libc::LOG_KERN,
            Facility::Local0 => libc::LOG_LOCAL0,
            Facility::Local1 => libc::LOG_LOCAL1,
            Facility::Local2 => libc::LOG_LOCAL2,
            Facility::Local3 => libc::LOG_LOCAL3,
            Facility::Local4 => libc::LOG_LOCAL4,
            Facility::Local5 => libc::LOG_LOCAL5,
            Facility::Local6 => libc::LOG_LOCAL6,
            Facility::Local7 => libc::LOG_LOCAL7,
            Facility::Lpr => libc::LOG_LPR,
            Facility::Mail => libc::LOG_MAIL,
            Facility::News => libc::LOG_NEWS,
            Facility::Syslog => libc::LOG_SYSLOG,
            Facility::User => libc::LOG_USER,
            Facility::Uucp => libc::LOG_UUCP,
        }
    }
}

pub struct SyslogAppenderBuilder {
    encoder: Option<Box<log4rs::encode::Encode>>,
    ident: Option<String>,
    level_map: Option<Box<LevelMap>>,
}

impl SyslogAppenderBuilder {
    pub fn encoder(mut self, encoder: Box<log4rs::encode::Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    pub fn openlog(mut self, ident: &str, option: LogOption, facility: Facility) -> Self {
        // At least on Linux openlog() does not copy this string, so we should keep it available.
        let mut ident = String::from(ident);
        ident.push('\0');
        unsafe {
            libc::openlog(
                ident.as_ptr() as *const libc::c_char,
                option.bits(),
                facility.into(),
            );
        }

        self.ident = Some(ident);
        self
    }

    pub fn level_map(mut self, level_map: Box<LevelMap>) -> Self {
        self.level_map = Some(level_map);
        self
    }

    pub fn build(self) -> SyslogAppender {
        SyslogAppender {
            encoder: self.encoder.unwrap_or_else(|| {
                Box::new(log4rs::encode::pattern::PatternEncoder::default())
            }),
            ident: self.ident,
            level_map: self.level_map,
        }
    }
}
