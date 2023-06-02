use std::fmt::{Display, Formatter};
use std::io::{Stderr, Stdout, Write};
use std::{fs, io, sync, thread};
use std::error::Error;
use std::fs::File;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use chrono::{Datelike, Timelike, Utc};
use once_cell::sync::Lazy;
use crate::core::Game;
use crate::settings::Setting::{Boolean, LogSeverity};
use crate::utils::logger::Severity::*;

pub static GLOBAL_LOGGER: Lazy<Arc<Logger>> = Lazy::new(|| Logger::new(sync::Weak::new()));

static MIN_SEVERITY: Severity = DEBUG;
static MAX_RECURSION_DEPTH: usize = 256;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum Severity {
    UNKNOWN,
    DEBUG,
    INFO,
    LOADING,
    WARNING,
    ERROR,
    FATAL
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UNKNOWN => write!(f, "UNKNOWN"),
            DEBUG => write!(f, "DEBUG"),
            INFO => write!(f, "INFO"),
            LOADING => write!(f, "LOADING"),
            WARNING => write!(f, "WARNING"),
            ERROR => write!(f, "ERROR"),
            FATAL => write!(f, "FATAL")
        }
    }
}

#[derive(Clone)]
struct LoggerData {
    print_severity: Severity,
    file_severity: Severity,
    log_file: Option<Arc<Mutex<File>>>,
    create_attempt: bool
}

impl LoggerData {

    fn new() -> Self {
        Self {
            print_severity: UNKNOWN,
            file_severity: UNKNOWN,
            log_file: None,
            create_attempt: false
        }
    }

}

pub struct Logger {
    game: sync::Weak<Game>,
    data: Mutex<LoggerData>,
    prefix: String,
    super_logger: sync::Weak<Logger>
}

impl Logger {

    pub fn new(game: sync::Weak<Game>) -> Arc<Self> {
        Arc::new(Self {
            game,
            data: Mutex::new(LoggerData::new()),
            prefix: String::new(),
            super_logger: sync::Weak::new()
        })
    }

    pub fn from_str(logger: &Arc<Logger>, prefix: &str) -> Arc<Self> {
        Logger::from_string(logger, prefix.to_string())
    }

    pub fn from_string(logger: &Arc<Logger>, prefix: String) -> Arc<Self> {
        Arc::new(Self {
            game: logger.game.clone(),
            data: Mutex::new(logger.data.lock().unwrap().clone()),
            prefix,
            super_logger: Arc::downgrade(logger)
        })
    }

    fn print(&self, message_severity: Severity, message: &str) {
        let mut data = self.data.lock().unwrap();
        // Severity is uninitialized
        if data.print_severity == UNKNOWN || data.file_severity == UNKNOWN {

            // Attempt to get the severity from the game settings
            if let Some(game) = self.game.upgrade() {
                // Print severity
                if let LogSeverity(print) = game.get_settings().get("log.printSeverity") {
                    data.print_severity = print;
                } else {
                    data.print_severity = MIN_SEVERITY;
                }

                // File severity
                if let LogSeverity(file) = game.get_settings().get("log.fileSeverity") {
                    data.file_severity = file;
                } else {
                    data.file_severity = MIN_SEVERITY;
                }
            } else {
                data.print_severity = MIN_SEVERITY;
                data.file_severity = MIN_SEVERITY;
            }
        }

        if message_severity >= data.print_severity {
            self.write_std(message_severity, message);
        }

        if message_severity >= data.file_severity {
            // Only if we haven't attempted to create the file yet...
            if !data.create_attempt {

                // ...and we have a valid game pointer...
                if let Some(game) = self.game.upgrade() {

                    // ...and this engine setting exists...
                    if let Boolean(create_log) = game.get_settings().get("log.autoCreate") {

                        // ...and the option tells us to create the file
                        if create_log {
                            self.create_log_file(&mut data);
                        }
                    }
                }
                data.create_attempt = true;
            }

            self.write_file(&mut data, message_severity, message);
        }

    }

    fn create_log_file(&self, data: &mut MutexGuard<LoggerData>) {
        // Try to create a logs folder
        match fs::create_dir_all("./logs") {
            Ok(_) => {},
            Err(error) => {
                Self::safe_print(true, "Error while creating folder structure for log files: ");
                Self::safe_println(true, Self::gen_error_str(&error).as_str());
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
                data.log_file = Some(Arc::new(Mutex::new(file)));
            },
            Err(error) => {
                data.log_file = None;
                Self::safe_print(true, "Error while creating log file: ");
                Self::safe_println(true, Self::gen_error_str(&error).as_str());
            }
        }
    }

    fn write_std(&self, message_severity: Severity, message: &str) {
        if message_severity >= ERROR {
            self.do_write(&mut io::stderr(), message_severity, message).unwrap_or(());
        } else {
            self.do_write(&mut io::stdout(), message_severity, message).unwrap_or(());
        }
    }

    fn write_file(&self, data: &mut MutexGuard<LoggerData>, message_severity: Severity, message: &str) {
        // Write to the file only if it's open
        if let Some(file_ptr) = &data.log_file {
            let mut file = file_ptr.lock().unwrap();

            let write_result = self.do_write(file.deref_mut(), message_severity, message);
            match write_result {
                Err(error) => {
                    // Write error, handle is probably dead, invalidate it
                    drop(file);
                    data.log_file = None;

                    Self::safe_print(true, "Cannot write to log file: ");
                    Self::safe_println(true, Self::gen_error_str(&error).as_str());
                },
                _ => {}
            }
        }
    }

    fn write_prefix<T>(&self, device: &mut T, depth: usize)
        -> io::Result<()>
        where T: Write {

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

    fn do_write<T>(&self, device: &mut T, message_severity: Severity, message: &str)
        -> io::Result<()>
        where T: Write {
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

        write!(device, "[{}/{}/{} {}:{}:{}.{}][{}",
                 date.day(),
                 date.month(),
                 date.year(),
                 date.hour(),
                 date.minute(),
                 date.second(),
                 date.timestamp_subsec_millis(),
                 if is_game {"GAME"} else {"GLOBAL"}
        )?;

        if is_game {
            write!(device, " {}", game_index)?;
        }

        write!(device, "][{}][{}]",
                thread_name,
                message_severity
        )?;

        self.write_prefix(device, 0)?;

        writeln!(device, ": {}", message)
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

    fn apply_to_current<F>(mut function: F) where F: FnMut(Arc<Logger>) {
        let game = Game::get_instance();
        if let Some(game) = game.upgrade() {
            function(game.get_logger());
        } else {
            function(GLOBAL_LOGGER.clone());
        }
    }

    pub fn get_print_severity(&self) -> Severity {
        self.data.lock().unwrap().print_severity
    }

    pub fn set_print_severity(&self, print_severity: Severity) {
        if let Some(game) = self.game.upgrade() {
            game.get_settings().set("log.printSeverity", LogSeverity(print_severity));
        } else {
            self.data.lock().unwrap().print_severity = print_severity;
        }
    }

    pub fn get_file_severity(&self) -> Severity {
        self.data.lock().unwrap().file_severity
    }

    pub fn set_file_severity(&self, print_severity: Severity) {
        if let Some(game) = self.game.upgrade() {
            game.get_settings().set("log.fileSeverity", LogSeverity(print_severity));
        } else {
            self.data.lock().unwrap().file_severity = print_severity;
        }
    }

    pub fn set_log_file(&self, file: Option<Arc<Mutex<File>>>) {
        self.data.lock().unwrap().log_file = file;
    }

    pub fn print_debug(&self, message: &str) {
        self.print(DEBUG, message);
    }

    pub fn print_debug_err(&self, message: &str, error: &dyn Error) {
        self.print(DEBUG, message);
        self.print(DEBUG, &Logger::gen_error_str(error));
    }

    pub fn print_info(&self, message: &str) {
        self.print(INFO, message);
    }

    pub fn print_info_err(&self, message: &str, error: &dyn Error) {
        self.print(INFO, message);
        self.print(INFO, &Logger::gen_error_str(error));
    }

    pub fn print_loading(&self, message: &str) {
        self.print(LOADING, message);
    }

    pub fn print_loading_err(&self, message: &str, error: &dyn Error) {
        self.print(LOADING, message);
        self.print(LOADING, &Logger::gen_error_str(error));
    }

    pub fn print_warning(&self, message: &str) {
        self.print(WARNING, message);
    }

    pub fn print_warning_err(&self, message: &str, error: &dyn Error) {
        self.print(WARNING, message);
        self.print(WARNING, &Logger::gen_error_str(error));
    }

    pub fn print_error(&self, message: &str) {
        self.print(ERROR, message);
    }

    pub fn print_error_err(&self, message: &str, error: &dyn Error) {
        self.print(ERROR, message);
        self.print(ERROR, &Logger::gen_error_str(error));
    }

    pub fn print_fatal(&self, message: &str) {
        self.print(FATAL, message);
    }

    pub fn print_fatal_err(&self, message: &str, error: &dyn Error) {
        self.print(FATAL, message);
        self.print(FATAL, &Logger::gen_error_str(error));
    }

    pub fn debug(message: &str) {
        Logger::apply_to_current(|logger| logger.print_debug(message));
    }

    pub fn debug_err(message: &str, error: &dyn Error) {
        Logger::apply_to_current(|logger| logger.print_debug_err(message, error));
    }

    pub fn info(message: &str) {
        Logger::apply_to_current(|logger| logger.print_info(message));
    }

    pub fn info_err(message: &str, error: &dyn Error) {
        Logger::apply_to_current(|logger| logger.print_info_err(message, error));
    }

    pub fn loading(message: &str) {
        Logger::apply_to_current(|logger| logger.print_loading(message));
    }

    pub fn loading_err(message: &str, error: &dyn Error) {
        Logger::apply_to_current(|logger| logger.print_loading_err(message, error));
    }

    pub fn warning(message: &str) {
        Logger::apply_to_current(|logger| logger.print_warning(message));
    }

    pub fn warning_err(message: &str, error: &dyn Error) {
        Logger::apply_to_current(|logger| logger.print_warning_err(message, error));
    }

    pub fn error(message: &str) {
        Logger::apply_to_current(|logger| logger.print_error(message));
    }

    pub fn error_err(message: &str, error: &dyn Error) {
        Logger::apply_to_current(|logger| logger.print_error_err(message, error));
    }

    pub fn fatal(message: &str) {
        Logger::apply_to_current(|logger| logger.print_fatal(message));
    }

    pub fn fatal_err(message: &str, error: &dyn Error) {
        Logger::apply_to_current(|logger| logger.print_fatal_err(message, error));
    }

    pub fn safe_print(error: bool, message: &str) {
        enum Type {
            OUT(Stdout),
            ERR(Stderr)
        }
        impl Type {
            fn as_writeable(&mut self) -> &mut dyn Write {
                match self {
                    Type::ERR(err) => &mut *err,
                    Type::OUT(out) => &mut *out
                }
            }
        }
        let mut device = if error { Type::ERR(io::stderr()) } else { Type::OUT(io::stdout()) };
        write!(device.as_writeable(), "{}", message).unwrap_or(());
    }

    pub fn safe_println(error: bool, message: &str) {
        Self::safe_print(error, message);
        Self::safe_print(error, "\n");
    }

}