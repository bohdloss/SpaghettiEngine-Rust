use std::fmt::{Display, Formatter};
use std::io::Write;
use std::{fs, sync, thread};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use chrono::{Datelike, Timelike, Utc};
use once_cell::sync::Lazy;
use crate::core::Game;
use crate::settings::Setting::{Boolean, LogSeverity};
use crate::utils::logger::Severity::*;

static GLOBAL_LOGGER: Lazy<Logger> = Lazy::new(|| Logger::new(sync::Weak::new()));

static MIN_SEVERITY: Severity = DEBUG;

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

    fn new(print_severity: Severity, file_severity: Severity) -> Self {
        Self {
            print_severity,
            file_severity,
            log_file: None,
            create_attempt: false
        }
    }

}

pub struct Logger {
    game: sync::Weak<Game>,
    super_prefix: String,
    super_logger: sync::Weak<Logger>,
    data: RwLock<LoggerData>
}

impl Logger {

    pub fn from(super_logger: sync::Weak<Logger>, prefix: &str) -> Self {
        // Clone some data from the super logger if the pointer is valid
        let game;
        let data;
        if let Some(logger) = super_logger.upgrade() {
            game = logger.game.clone();
            let guard = logger.data.read().unwrap();
            data = guard.clone();
        } else {
            game = sync::Weak::new();
            data = LoggerData::new(UNKNOWN, UNKNOWN);
        }

        Self {
            game,
            super_prefix: format!("[{}]", prefix).to_string(),
            super_logger,
            data: RwLock::new(data)
        }
    }

    pub fn new(game: sync::Weak<Game>) -> Self {
        Self {
            game,
            super_prefix: "".to_string(),
            super_logger: sync::Weak::new(),
            data: RwLock::new(LoggerData::new(UNKNOWN, UNKNOWN))
        }
    }

    fn print(&self, message_severity: Severity, message: &str) {
        if let Some(super_logger) = self.super_logger.upgrade() {
            super_logger.print(message_severity, format!("{} {}", self.super_prefix, message).as_str());
            return;
        }

        let severity = self.data.read().unwrap();
        // Severity is uninitialized
        if severity.print_severity == UNKNOWN || severity.file_severity == UNKNOWN {
            drop(severity);
            let mut severity = self.data.write().unwrap();

            // Only update severity if we have a valid game pointer
            if let Some(game) = self.game.upgrade() {
                // Print severity
                if let LogSeverity(print) = game.get_settings().get("log.printSeverity") {
                    severity.print_severity = print;
                }

                // File severity
                if let LogSeverity(file) = game.get_settings().get("log.fileSeverity") {
                    severity.file_severity = file;
                }
            } else {
                severity.print_severity = MIN_SEVERITY;
                severity.file_severity = MIN_SEVERITY;
            }
        } else {
            drop(severity);
        }

        if message_severity >= self.data.read().unwrap().print_severity {
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


            if message_severity >= ERROR {
                eprintln!("[{}/{}/{} {}:{}:{}.{}][{}{}][{}][{}]: {}",
                          date.day(),
                          date.month(),
                          date.year(),
                          date.hour(),
                          date.minute(),
                          date.second(),
                          date.timestamp_subsec_millis(),
                          if is_game {"GAME "} else {"GLOBAL "},
                          if is_game {game_index} else {0},
                          thread_name,
                          message_severity,
                          message);
            } else {
                println!("[{}/{}/{} {}:{}:{}.{}][{}{}][{}][{}]: {}",
                         date.day(),
                         date.month(),
                         date.year(),
                         date.hour(),
                         date.minute(),
                         date.second(),
                         date.timestamp_subsec_millis(),
                    if is_game {"GAME "} else {"GLOBAL "},
                    if is_game {game_index} else {0},
                    thread_name,
                    message_severity,
                    message);
            }
        }

        // Only if we haven't attempted to create the file yet...
        if !self.data.read().unwrap().create_attempt {

            // ...and we have a valid game pointer...
            if let Some(game) = self.game.upgrade() {

                // ...and this engine setting exists...
                if let Boolean(create_log) = game.get_settings().get("log.autoCreate") {

                    // ...and the option tells us to create the file
                    if create_log {
                        self.create_log_file();
                    }
                }
            }
        }

        let mut data = self.data.write().unwrap();
        data.create_attempt = true;

        // Write to the file only if it's open
        if let Some(file_ptr) = &data.log_file {
            let mut file = file_ptr.lock().unwrap();

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

            let write_result = writeln!(file, "[{}/{}/{} {}:{}:{}.{}][{}{}][{}][{}]: {}",
                      date.day(),
                      date.month(),
                      date.year(),
                      date.hour(),
                      date.minute(),
                      date.second(),
                      date.timestamp_subsec_millis(),
                      if is_game {"GAME "} else {"GLOBAL "},
                      if is_game {game_index} else {0},
                      thread_name,
                      message_severity,
                      message);

            match write_result {
                Err(error) => {
                    // Write error, handle is probably dead, invalidate it
                    drop(file);
                    data.log_file = None;

                    eprintln!("Cannot write to log file: {}", error)
                },
                _ => {}
            }
        }
    }

    fn create_log_file(&self) {
        // Try to create a logs folder
        match fs::create_dir_all("./logs") {
            Ok(_) => {},
            Err(error) => {
                eprintln!("Error while creating folder structure for log files: {}", error);
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

        let mut data = self.data.write().unwrap();
        match File::create(log_file_path) {
            Ok(file) => {
                // Update the log file
                data.log_file = Some(Arc::new(Mutex::new(file)));
            },
            Err(error) => {
                data.log_file = None;
                eprintln!("Error while creating log file: {}", error);
            }
        }
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

    fn apply_to_current<F>(function: F) where F: Fn(&Logger) {
        let game = Game::get_instance();
        if let Some(game) = game.upgrade() {
            function(game.get_logger());
        } else {
            function(&GLOBAL_LOGGER);
        }
    }

    pub fn get_print_severity(&self) -> Severity {
        self.data.read().unwrap().print_severity
    }

    pub fn set_print_severity(&self, print_severity: Severity) {
        if let Some(game) = self.game.upgrade() {
            game.get_settings().set("log.printSeverity", LogSeverity(print_severity));
        } else {
            self.data.write().unwrap().print_severity = print_severity;
        }
    }

    pub fn get_file_severity(&self) -> Severity {
        self.data.read().unwrap().file_severity
    }

    pub fn set_file_severity(&self, print_severity: Severity) {
        if let Some(game) = self.game.upgrade() {
            game.get_settings().set("log.fileSeverity", LogSeverity(print_severity));
        } else {
            self.data.write().unwrap().file_severity = print_severity;
        }
    }

    pub fn set_log_file(&self, file: Option<Arc<Mutex<File>>>) {
        self.data.write().unwrap().log_file = file;
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

}