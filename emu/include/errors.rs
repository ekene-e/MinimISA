use std::sync::{Arc, Mutex};
use std::fmt;
use std::process;

/// Error levels similar to the C `error_t` enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorLevel {
    Note,      // Output contents to stderr
    Warn,      // Warn and continue execution
    Error,     // Print error and continue execution
    IError,    // Display internal error and continue
    Fatal,     // Display fatal error and exit(1)
    IFatal,    // Display internal error and exit(1)
}

/// ErrorFlag structure to manage the error flag
pub struct ErrorFlag {
    flag: Arc<Mutex<bool>>,  
}

impl ErrorFlag {
    pub fn new() -> Self {
        ErrorFlag {
            flag: Arc::new(Mutex::new(false)),
        }
    }

    pub fn error_msg(&self, level: ErrorLevel, format: &str, args: fmt::Arguments) {
        match level {
            ErrorLevel::Note => {
                eprintln!("note: {}", format);
            }
            ErrorLevel::Warn => {
                eprintln!("warning: {}", format);
            }
            ErrorLevel::Error | ErrorLevel::IError => {
                eprintln!("error: {}", format);
                *self.flag.lock().unwrap() = true;
            }
            ErrorLevel::Fatal => {
                eprintln!("fatal: {}", format);
                process::exit(1);
            }
            ErrorLevel::IFatal => {
                eprintln!("internal fatal error: {}", format);
                process::exit(1);
            }
        }
    }

    pub fn error_msg_fmt(&self, level: ErrorLevel, format: &str, args: fmt::Arguments) {
        self.error_msg(level, format, args);
    }

    pub fn clear(&self) {
        *self.flag.lock().unwrap() = false;
    }

    pub fn check(&self) {
        if *self.flag.lock().unwrap() {
            eprintln!("Error flag is set, exiting.");
            process::exit(1);
        }
    }
}

/// Convenience macros to emit specific types of errors
#[macro_export]
macro_rules! note {
    ($error_flag:expr, $($arg:tt)*) => {
        $error_flag.error_msg_fmt(ErrorLevel::Note, &format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($error_flag:expr, $($arg:tt)*) => {
        $error_flag.error_msg_fmt(ErrorLevel::Warn, &format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($error_flag:expr, $($arg:tt)*) => {
        $error_flag.error_msg_fmt(ErrorLevel::Error, &format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! ierror {
    ($error_flag:expr, $($arg:tt)*) => {
        $error_flag.error_msg_fmt(ErrorLevel::IError, &format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! fatal {
    ($error_flag:expr, $($arg:tt)*) => {
        $error_flag.error_msg_fmt(ErrorLevel::Fatal, &format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! ifatal {
    ($error_flag:expr, $($arg:tt)*) => {
        $error_flag.error_msg_fmt(ErrorLevel::IFatal, &format_args!($($arg)*));
    };
}
