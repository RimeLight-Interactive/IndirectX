use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    tx: Option<mpsc::Sender<String>>,
    file: Option<File>,
    enabled: bool,
    async_mode: bool,
}

impl Logger {
    pub fn create<P: AsRef<Path>>(path: P, enabled: bool, async_mode: bool) -> io::Result<Self> {
        if !enabled {
            return Ok(Self { tx: None, file: None, enabled: false, async_mode });
        }

        if async_mode {
            let mut file = OpenOptions::new().create(true).truncate(true).write(true).open(path)?;
            let (tx, rx) = mpsc::channel::<String>();
            
            thread::spawn(move || {
                while let Ok(msg) = rx.recv() {
                    let _ = writeln!(file, "{} {}", current_timestamp(), msg);
                    let _ = file.flush();
                }
            });
            Ok(Self { tx: Some(tx), file: None, enabled: true, async_mode: true })
        } else {
            let file = OpenOptions::new().create(true).truncate(true).write(true).open(path)?;
            Ok(Self { tx: None, file: Some(file), enabled: true, async_mode: false })
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        if !self.enabled { return; }

        if self.async_mode {
            if let Some(tx) = &self.tx {
                let _ = tx.send(format!("{}", args));
            }
        } else if let Some(file) = &mut self.file {
            let _ = writeln!(file, "{} {}", current_timestamp(), args);
            let _ = file.flush();
        }
    }

    pub fn log(&mut self, message: &str) {
        self.write_fmt(format_args!("{}", message));
    }
}

static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

pub fn init<P: AsRef<Path>>(path: P, enabled: bool, async_mode: bool) -> io::Result<()> {
    let logger = Logger::create(path, enabled, async_mode)?;
    LOGGER
        .set(Mutex::new(logger))
        .map_err(|_| io::Error::new(io::ErrorKind::AlreadyExists, "Logger already initialized"))?;
    Ok(())
}

pub fn enabled() -> bool {
    LOGGER.get().map(|l| l.lock().unwrap().enabled()).unwrap_or(false)
}

pub fn write_fmt(args: fmt::Arguments) {
    if let Some(mutex) = LOGGER.get() {
        if let Ok(mut logger) = mutex.lock() {
            logger.write_fmt(args);
        }
    }
}

pub fn log(message: &str) {
    if let Some(mutex) = LOGGER.get() {
        if let Ok(mut logger) = mutex.lock() {
            logger.log(message);
        }
    }
}

fn current_timestamp() -> String {
    let now = SystemTime::now();
    let elapsed = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}.{}", elapsed.as_secs(), elapsed.subsec_millis())
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)+) => {
        if $crate::logger::enabled() {
            $crate::logger::write_fmt(format_args!($($arg)+));
        }
    };
}
