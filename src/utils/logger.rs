use crate::core::Game;
use crate::settings::Setting::LogSeverity;
use crate::utils::logger::Severity::*;
use chrono::{Datelike, Timelike, Utc};
use once_cell::sync::Lazy;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, io, sync, thread};

pub static GLOBAL_LOGGER: Lazy<Arc<Logger>> = Lazy::new(|| Logger::new(sync::Weak::new()));

static MIN_SEVERITY: Severity = Debug;
static MAX_RECURSION_DEPTH: usize = 256;

#[macro_export]
macro_rules! log {
	($severity: ident, $error:expr, $format:literal, $($arg:expr),*) => {{
        use $crate::utils::logger::Severity::*;
		$crate::utils::logger::Logger::log_err($severity, |device| {
			write!(device, $format, $($arg),*)
		}, $error);
	}};
	($severity: ident, $format:literal, $($arg:expr),*) => {{
        use $crate::utils::logger::Severity::*;
		$crate::utils::logger::Logger::log($severity, |device| {
			write!(device, $format, $($arg),*)
		});
	}};
    ($severity: ident, $error:expr, $format:literal) => {{
        use $crate::utils::logger::Severity::*;
		$crate::utils::logger::Logger::log_err($severity, |device| {
			write!(device, $format)
		}, $error);
	}};
	($severity: ident, $format:literal) => {{
        use $crate::utils::logger::Severity::*;
		$crate::utils::logger::Logger::log($severity, |device| {
			write!(device, $format)
		});
	}};
    ($logger:expr, $severity:ident, $error:expr, $format:literal, $($arg:expr),*) => {{
        use $crate::utils::logger::Severity::*;
		$logger.print_err($severity, |device| {
			write!(device, $format, $($arg),*)
		}, $error);
	}};
	($logger:expr, $severity:ident, $format:literal, $($arg:expr),*) => {{
        use $crate::utils::logger::Severity::*;
		$logger.print($severity, |device| {
			write!(device, $format, $($arg),*)
		});
	}};
    ($logger:expr, $severity:ident, $error:expr, $format:literal) => {{
        use $crate::utils::logger::Severity::*;
		$logger.print_err($severity, |device| {
			write!(device, $format)
		}, $error);
	}};
	($logger:expr, $severity:ident, $format:literal) => {{
        use $crate::utils::logger::Severity::*;
		$logger.print($severity, |device| {
			write!(device, $format)
		});
	}};
}

#[macro_export]
macro_rules! safe_print {
    ($format:literal, $($arg:expr),*) => {
         write!(io::stdout(), $format, $($arg),*).unwrap_or(());
    };
    ($format:literal) => {
        write!(io::stdout(), $format).unwrap_or(());
    }
}

#[macro_export]
macro_rules! safe_println {
    ($format:literal, $($arg:expr),*) => {
         writeln!(io::stdout(), $format, $($arg),*).unwrap_or(());
    };
    ($format:literal) => {
        writeln!(io::stdout(), $format).unwrap_or(());
    }
}

#[macro_export]
macro_rules! safe_eprint {
    ($format:literal, $($arg:expr),*) => {
         write!(io::stderr(), $format, $($arg),*).unwrap_or(());
    };
    ($format:literal) => {
        write!(io::stderr(), $format).unwrap_or(());
    }
}

#[macro_export]
macro_rules! safe_eprintln {
    ($format:literal, $($arg:expr),*) => {
         writeln!(io::stderr(), $format, $($arg),*).unwrap_or(());
    };
    ($format:literal) => {
        writeln!(io::stderr(), $format).unwrap_or(());
    }
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd)]
/// Represents the severity of a logger message,
/// and it will be displayed and used to determine
/// if a message should be printed or not
pub enum Severity {
    Unknown,
    Debug,
    Info,
    Loading,
    Warning,
    Error,
    Fatal,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unknown => write!(f, "Unknown"),
            Debug => write!(f, "Debug"),
            Info => write!(f, "Info"),
            Loading => write!(f, "Loading"),
            Warning => write!(f, "Warning"),
            Error => write!(f, "Error"),
            Fatal => write!(f, "Fatal"),
        }
    }
}

struct LoggerData {
    print_severity: Severity,
    file_severity: Severity,
    log_file: Option<File>,
    create_attempt: bool,
}

impl LoggerData {
    fn new() -> Self {
        Self {
            print_severity: Unknown,
            file_severity: Unknown,
            log_file: None,
            create_attempt: false,
        }
    }
}

/// A logger allows you to print messages to stdout in a
/// standardized way while also optionally logging them to a file
///
/// Unlike the print macro, a logger will not panic on an error but rather
/// ignore it and, if the error happens while writing to a file, close the handle
pub struct Logger {
    game: sync::Weak<Game>,
    data: Arc<Mutex<LoggerData>>,
    prefix: String,
    super_logger: sync::Weak<Logger>,
}

impl Logger {
    /// Creates a new logger for a given game instance
    ///
    /// # Arguments
    /// * `game` - The game instance
    ///
    /// # Returns
    /// * A new logger
    pub fn new(game: sync::Weak<Game>) -> Arc<Self> {
        Arc::new(Self {
            game,
            data: Arc::new(Mutex::new(LoggerData::new())),
            prefix: String::new(),
            super_logger: sync::Weak::new(),
        })
    }

    /// Creates a sub-logger of the given logger.
    ///
    /// When printing with this logger, it will act
    /// exactly like its parent but will also print
    /// the given prefix.
    ///
    /// Any change done to this logger will reflect on
    /// the parent and vice versa
    ///
    /// # Arguments
    /// - `logger` - The parent logger
    /// - `prefix` - The prefix of this logger as a string literal
    pub fn from_str(logger: &Arc<Logger>, prefix: &str) -> Arc<Self> {
        Logger::from_string(logger, prefix.to_string())
    }

    /// Creates a sub-logger of the given logger.
    ///
    /// When printing with this logger, it will act
    /// exactly like its parent but will also print
    /// the given prefix.
    ///
    /// Any change done to this logger will reflect on
    /// the parent and vice versa
    ///
    /// # Arguments
    /// - `logger` - The parent logger
    /// - `prefix` - The prefix of this logger as a String
    pub fn from_string(logger: &Arc<Logger>, prefix: String) -> Arc<Self> {
        Arc::new(Self {
            game: logger.game.clone(),
            data: logger.data.clone(),
            prefix,
            super_logger: Arc::downgrade(logger),
        })
    }

    fn do_print<T>(&self, message_severity: Severity, message: &T, error: Option<&dyn Error>)
    where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        let mut data = self.data.lock().unwrap();
        // Severity is uninitialized
        if data.print_severity == Unknown || data.file_severity == Unknown {
            // Attempt to get the severity from the game settings
            if let Some(game) = self.game.upgrade() {
                // Print severity
                data.print_severity = game
                    .get_settings()
                    .get("log.printSeverity")
                    .as_log_severity_or(MIN_SEVERITY);

                // File severity
                data.file_severity = game
                    .get_settings()
                    .get("log.fileSeverity")
                    .as_log_severity_or(MIN_SEVERITY);
            } else {
                data.print_severity = MIN_SEVERITY;
                data.file_severity = MIN_SEVERITY;
            }
        }

        if message_severity >= data.print_severity {
            self.write_std(message_severity, message, error);
        }

        if message_severity >= data.file_severity {
            // Only if we haven't attempted to create the file yet...
            if !data.create_attempt {
                // ...and we have a valid game pointer...
                if let Some(game) = self.game.upgrade() {
                    // ...and the option tells us to create the file
                    if game
                        .get_settings()
                        .get("log.autoCreate")
                        .as_boolean_or(false)
                    {
                        self.create_log_file(&mut data);
                    }
                }
                data.create_attempt = true;
            }

            self.write_file(&mut data, message_severity, message, error);
        }
    }

    fn create_log_file(&self, data: &mut MutexGuard<LoggerData>) {
        // Try to create a logs folder
        match fs::create_dir_all("./logs") {
            Ok(_) => {}
            Err(error) => {
                safe_eprint!("Error while creating folder structure for log files: ");
                safe_eprintln!("{}", Self::gen_error_str(&error).as_str());
                return;
            }
        }
        let date = Utc::now();

        // Base file name
        let name = format!("./logs/{}.log", date.format("%d_%m_%Y_%H_%M"));
        let mut name2: String;
        let mut log_file_path = Path::new(&name);

        // Keep trying increasing log file indexes until we find a free file name
        let mut number: usize = 0;
        while log_file_path.exists() {
            number += 1;
            name2 = format!("./logs/{}_{}.log", date.format("%d_%m_%Y_%H_%M"), number);
            log_file_path = Path::new(&name2);
        }

        match File::create(log_file_path) {
            Ok(file) => {
                // Update the log file
                data.log_file = Some(file);
            }
            Err(error) => {
                data.log_file = None;
                safe_eprint!("Error while creating log file: ");
                safe_eprintln!("{}", Self::gen_error_str(&error).as_str());
            }
        }
    }

    fn write_std<T>(&self, message_severity: Severity, message: &T, error: Option<&dyn Error>)
    where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        if message_severity >= Error {
            self.do_write(&mut io::stderr(), message_severity, message, error)
                .unwrap_or(());
        } else {
            self.do_write(&mut io::stdout(), message_severity, message, error)
                .unwrap_or(());
        }
    }

    fn write_file<T>(
        &self,
        data: &mut MutexGuard<LoggerData>,
        message_severity: Severity,
        message: &T,
        error: Option<&dyn Error>,
    ) where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        // Write to the file only if it's open
        if let Some(file) = &mut data.log_file {
            let write_result = self.do_write(file, message_severity, message, error);
            match write_result {
                Err(error) => {
                    // Write error, handle is probably dead, invalidate it
                    data.log_file = None;

                    safe_eprint!("Cannot write to log file: ");
                    safe_eprintln!("{}", Self::gen_error_str(&error).as_str());
                }
                _ => {}
            }
        }
    }

    fn write_prefix<T>(&self, device: &mut T, depth: usize) -> io::Result<()>
    where
        T: Write,
    {
        // Failsafe for too many nested loggers
        if depth >= MAX_RECURSION_DEPTH {
            return Ok(());
        }

        if let Some(super_logger) = self.super_logger.upgrade() {
            super_logger.write_prefix(device, depth + 1)?;
        }

        if self.prefix != "" {
            write!(device, "[{}]", self.prefix)?;
        }
        Ok(())
    }

    fn write_complete_prefix<T>(&self, device: &mut T, message_severity: Severity) -> io::Result<()>
    where
        T: Write,
    {
        // Retrieve some data
        let is_game;
        let game_index;
        let thread = thread::current();
        let thread_name = thread.name().unwrap_or("*unnamed_thread*");
        let date = Utc::now();

        if let Some(game) = self.game.upgrade() {
            is_game = true;
            game_index = game.get_index();
        } else {
            is_game = false;
            game_index = 0;
        }

        write!(
            device,
            "[{}/{}/{} {}:{}:{}.{}][{}",
            date.day(),
            date.month(),
            date.year(),
            date.hour(),
            date.minute(),
            date.second(),
            date.timestamp_subsec_millis(),
            if is_game { "GAME" } else { "GLOBAL" }
        )?;

        if is_game {
            write!(device, " {}", game_index)?;
        }

        write!(device, "][{}][{}]", thread_name, message_severity)?;

        self.write_prefix(device, 0)?;

        write!(device, ": ")
    }

    fn do_write<T, F>(
        &self,
        device: &mut T,
        message_severity: Severity,
        message: &F,
        error: Option<&dyn Error>,
    ) -> io::Result<()>
    where
        T: Write,
        F: Fn(&mut dyn Write) -> io::Result<()>,
    {
        self.write_complete_prefix(device, message_severity)?;

        message(device)?;
        writeln!(device)?;

        // Write errors
        if let Some(error) = error {
            self.write_complete_prefix(device, message_severity)?;
            writeln!(device, "Error: {}", error)?;

            let mut source_option = error.source();
            let mut depth: usize = 0;
            while let Some(source) = source_option {
                if depth >= MAX_RECURSION_DEPTH {
                    break;
                }

                self.write_complete_prefix(device, message_severity)?;
                writeln!(device, "Caused by: {}", source)?;
                source_option = source.source();

                depth += 1;
            }
        }

        Ok(())
    }

    fn gen_error_str(error: &dyn Error) -> String {
        let mut err_str = format!("Error: {}", error);
        let mut source_option = error.source();
        while let Some(source) = source_option {
            let to_push = format!("\nCaused by: {}", source);
            err_str.push_str(&to_push);
            source_option = source.source();
        }
        err_str
    }

    fn apply_to_current<F>(function: F)
    where
        F: FnOnce(Arc<Logger>),
    {
        let game = Game::get_instance();
        if let Some(game) = game.upgrade() {
            function(game.get_logger());
        } else {
            function(GLOBAL_LOGGER.clone());
        }
    }

    /// Retrieves the minimum severity this logger
    /// requires for messages to be printed to
    /// standard output
    ///
    /// # Returns
    /// * The print severity
    pub fn get_print_severity(&self) -> Severity {
        self.data.lock().unwrap().print_severity
    }

    /// Changes the game setting for the minimum
    /// severity for messages to be printed to
    /// standard output
    ///
    /// # Arguments
    /// * `print_severity` - The new severity
    pub fn set_print_severity(&self, print_severity: Severity) {
        if let Some(game) = self.game.upgrade() {
            game.get_settings()
                .set("log.printSeverity", LogSeverity(print_severity));
        } else {
            self.data.lock().unwrap().print_severity = print_severity;
        }
    }

    /// Retrieves the minimum severity this logger
    /// requires for messages to be written to file
    ///
    /// # Returns
    /// * The print severity
    pub fn get_file_severity(&self) -> Severity {
        self.data.lock().unwrap().file_severity
    }

    /// Changes the game setting for the minimum
    /// severity for messages to be written to file
    ///
    /// # Arguments
    /// * `file_severity` - The new severity
    pub fn set_file_severity(&self, file_severity: Severity) {
        if let Some(game) = self.game.upgrade() {
            game.get_settings()
                .set("log.fileSeverity", LogSeverity(file_severity));
        } else {
            self.data.lock().unwrap().file_severity = file_severity;
        }
    }

    /// Changes the file log messages are written to
    ///
    /// # Arguments
    /// * `file` - The new file (or no file)
    pub fn set_log_file(&self, file: Option<File>) {
        self.data.lock().unwrap().log_file = file;
    }
    /// Logs a message with the specified severity
    ///
    /// # Arguments
    /// * `severity` - The severity of the message
    /// * `message` - The message to print
    pub fn print<T>(&self, severity: Severity, message: T)
    where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        self.do_print(severity, &message, None);
    }

    /// Logs a message with an error with the specified severity
    ///
    /// # Arguments
    /// * `severity` - The severity of the message
    /// * `message` - The message to print
    pub fn print_err<T>(&self, severity: Severity, message: T, error: &dyn Error)
    where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        self.do_print(severity, &message, Some(error));
    }

    /// Logs a message with the specified severity
    ///
    /// # Arguments
    /// * `severity` - The severity of the message
    /// * `message` - The message to print
    pub fn log<T>(severity: Severity, message: T)
    where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        Self::apply_to_current(|logger| logger.print(severity, message));
    }

    /// Logs a message with an error with the specified severity
    ///
    /// # Arguments
    /// * `severity` - The severity of the message
    /// * `message` - The message to print
    pub fn log_err<T>(severity: Severity, message: T, error: &dyn Error)
    where
        T: Fn(&mut dyn Write) -> io::Result<()>,
    {
        Self::apply_to_current(|logger| logger.print_err(severity, message, error));
    }
}
