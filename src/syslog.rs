use std;

use libc;
use log;
use log4rs;
#[cfg(feature = "file")]
use serde;

const DEFAULT_BUF_SIZE: usize = 4096;

type PersistentBuf = std::io::Cursor<Vec<u8>>;

thread_local! {
    static PERSISTENT_BUF: std::cell::RefCell<PersistentBuf> =
        std::cell::RefCell::new(PersistentBuf::new(Vec::with_capacity(DEFAULT_BUF_SIZE)));
}

struct BufWriter {}

impl BufWriter {
    fn new() -> Self {
        PERSISTENT_BUF.with(|pers_buf| pers_buf.borrow_mut().set_position(0));
        Self {}
    }

    fn as_c_str(&mut self) -> *const libc::c_char {
        use std::io::Write;

        PERSISTENT_BUF.with(|pers_buf| {
            let mut pers_buf = pers_buf.borrow_mut();
            pers_buf.write_all(&[0; 1]).unwrap();
            pers_buf.get_ref().as_ptr() as *const libc::c_char
        })
    }
}

impl std::io::Write for BufWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        PERSISTENT_BUF.with(|pers_buf| pers_buf.borrow_mut().write(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        PERSISTENT_BUF.with(|pers_buf| pers_buf.borrow_mut().flush())
    }
}

impl log4rs::encode::Write for BufWriter {}

/// Function for mapping rust's `log` levels to `libc`'s log levels.
pub type LevelMap = Fn(log::Level) -> libc::c_int + Send + Sync;

/// An appender which writes log invents into syslog using `libc`'s syslog() function.
pub struct SyslogAppender {
    encoder: Box<log4rs::encode::Encode>,
    level_map: Option<Box<LevelMap>>,
}

impl std::fmt::Debug for SyslogAppender {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "SyslogAppender {{encoder: {:?}, level_map: {}}}",
            self.encoder,
            match self.level_map {
                Some(_) => "Some(_)",
                None => "None",
            }
        )
    }
}

impl SyslogAppender {
    /// Create new builder for `SyslogAppender`.
    pub fn builder() -> SyslogAppenderBuilder {
        SyslogAppenderBuilder {
            encoder: None,
            openlog_args: None,
            level_map: None,
        }
    }
}

impl log4rs::append::Append for SyslogAppender {
    fn append(&self, record: &log::Record) -> std::result::Result<(), Box<std::error::Error + Sync + Send>> {
        let mut buf = BufWriter::new();

        self.encoder.encode(&mut buf, record)?;

        let level = match self.level_map {
            Some(ref level_map) => level_map(record.level()),

            None => match record.level() {
                log::Level::Error => libc::LOG_ERR,
                log::Level::Warn => libc::LOG_WARNING,
                log::Level::Info => libc::LOG_INFO,
                log::Level::Debug | log::Level::Trace => libc::LOG_DEBUG,
            },
        };

        unsafe {
            // This function may use the `ident` pointer previously set by `libc::openlog()`, until the call to
            // `libc::closelog()`.
            libc::syslog(
                level,
                b"%s\0".as_ptr() as *const libc::c_char,
                buf.as_c_str(),
            );
        }

        Ok(())
    }

    fn flush(&self) {}
}

bitflags! {
    /// Syslog option flags.
    pub struct LogOption: libc::c_int {
        /// Write directly to system console if there is an error while sending to system logger.
        const LOG_CONS   = libc::LOG_CONS;
        /// Open the connection immediately (normally, the connection is opened when the first message is logged).
        const LOG_NDELAY = libc::LOG_NDELAY;
        /// Don't wait for child processes that may have been created while logging the message.
        /// The GNU C library does not create a child process, so this option has no effect on Linux.
        const LOG_NOWAIT = libc::LOG_NOWAIT;
        /// The converse of LOG_NDELAY; opening of the connection is delayed until syslog() is called.
        /// This is the default, and need not be specified.
        const LOG_ODELAY = libc::LOG_ODELAY;
        /// Print to stderr as well. (Not in POSIX.1-2001 or POSIX.1-2008.)
        const LOG_PERROR = libc::LOG_PERROR;
        /// Include PID with each message.
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
                    "LOG_CONS" => flags |= LogOption::LOG_CONS,
                    "LOG_NDELAY" => flags |= LogOption::LOG_NDELAY,
                    "LOG_NOWAIT" => flags |= LogOption::LOG_NOWAIT,
                    "LOG_ODELAY" => flags |= LogOption::LOG_ODELAY,
                    "LOG_PERROR" => flags |= LogOption::LOG_PERROR,
                    "LOG_PID" => flags |= LogOption::LOG_PID,
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

#[derive(Debug)]
#[cfg_attr(feature = "file", derive(Deserialize))]
/// The type of program.
pub enum Facility {
    /// Security/authorization.
    Auth,
    /// Security/authorization (private).
    AuthPriv,
    /// Clock daemon (cron and at).
    Cron,
    /// System daemons without separate facility value.
    Daemon,
    /// FTP daemon.
    Ftp,
    /// Kernel messages (these can't be generated from user processes).
    Kern,
    /// Reserved for local use.
    Local0,
    /// Reserved for local use.
    Local1,
    /// Reserved for local use.
    Local2,
    /// Reserved for local use.
    Local3,
    /// Reserved for local use.
    Local4,
    /// Reserved for local use.
    Local5,
    /// Reserved for local use.
    Local6,
    /// Reserved for local use.
    Local7,
    /// Line printer subsystem.
    Lpr,
    /// Mail subsystem.
    Mail,
    /// USENET news subsystem.
    News,
    /// Messages generated internally by syslogd.
    Syslog,
    /// Generic user-level messages. This is the default when not calling openlog().
    User,
    /// UUCP subsystem.
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

struct OpenLogArgs {
    ident: String,
    log_option: LogOption,
    facility: Facility,
}

struct IdentHolder {
    ident: Option<String>,
}

impl IdentHolder {
    fn new() -> Self {
        Self { ident: None }
    }

    fn openlog(&mut self, mut args: OpenLogArgs) {
        args.ident.push('\0');

        unsafe {
            // This globally sets the `ident` pointer, which may be used by subsequent calls to the `libc::syslog()`
            // function. Pointer should remain valid and unchanged until either call to `libc::closelog()` or call
            // to `libc::openlog()` with new value of `ident`.
            libc::openlog(
                args.ident.as_ptr() as *const libc::c_char,
                args.log_option.bits(),
                args.facility.into(),
            );
        }

        // At least on Linux openlog() does not copy this string, so we should keep it available.
        self.ident = Some(args.ident);
    }

    fn closelog(&mut self) {
        if self.ident.is_some() {
            unsafe {
                // Among other things, this call discards the `ident` pointer set by `libc::openlog()`.
                // After this call, `ident` may be safely dropped.
                libc::closelog();
            }
        }
    }

    fn no_openlog(&mut self) {
        self.closelog();
        self.ident = None;
    }
}

impl Drop for IdentHolder {
    fn drop(&mut self) {
        // Currently this function is never used automatically because IdentHolder is created only by lazy_static.
        self.closelog();
    }
}

lazy_static! {
    static ref IDENT_HOLDER: std::sync::Mutex<IdentHolder> = std::sync::Mutex::new(IdentHolder::new());
}

/// Builder for `SyslogAppender`.
pub struct SyslogAppenderBuilder {
    encoder: Option<Box<log4rs::encode::Encode>>,
    openlog_args: Option<OpenLogArgs>,
    level_map: Option<Box<LevelMap>>,
}

impl SyslogAppenderBuilder {
    /// Set custom encoder.
    pub fn encoder(mut self, encoder: Box<log4rs::encode::Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    /// Call openlog().
    pub fn openlog(mut self, ident: &str, option: LogOption, facility: Facility) -> Self {
        self.openlog_args = Some(OpenLogArgs {
            ident: String::from(ident),
            log_option: option,
            facility,
        });
        self
    }

    /// Set custom log level mapping.
    pub fn level_map(mut self, level_map: Box<LevelMap>) -> Self {
        self.level_map = Some(level_map);
        self
    }

    /// Consume builder and produce `SyslogAppender`.
    pub fn build(self) -> SyslogAppender {
        self.openlog_args.map_or_else(
            || IDENT_HOLDER.lock().unwrap().no_openlog(),
            |openlog_args| IDENT_HOLDER.lock().unwrap().openlog(openlog_args),
        );

        SyslogAppender {
            encoder: self.encoder
                .unwrap_or_else(|| Box::new(log4rs::encode::pattern::PatternEncoder::default())),
            level_map: self.level_map,
        }
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    use test;

    fn bench(bencher: &mut test::Bencher, data: &[u8]) {
        use std::io::Write;

        bencher.iter(|| {
            let mut buf = super::BufWriter::new();
            buf.write_all(data).unwrap();
            buf.as_c_str()
        })
    }

    #[bench]
    fn buf_writer_no_realloc(bencher: &mut test::Bencher) {
        bench(bencher, &[b'x'; super::DEFAULT_BUF_SIZE - 1])
    }

    #[bench]
    fn buf_writer_realloc(bencher: &mut test::Bencher) {
        bench(bencher, &[b'x'; super::DEFAULT_BUF_SIZE])
    }
}
