use crate::log;
use crate::utils::{logger, Logger};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
struct DummyError {}

impl Display for DummyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "This is dummy error message")
    }
}

impl Error for DummyError {}

#[test]
fn log_macro() {
    // This is a compilation test more than anything

    let logger = Logger::from_str(&logger::GLOBAL_LOGGER, "TestSubLogger");

    let arg1: bool = false;
    let arg2: i32 = 42;
    let arg3: &str = "HELLO THERE";

    let error = DummyError {};

    // Severity, error, literal, format args
    log!(Warning, &error, "Formatting: {}, {}, {}", arg1, arg2, arg3);

    // Severity, literal, format args
    log!(Warning, "Another formatting: {}, {}, {}", arg1, arg2, arg3);

    // Severity, error, literal
    log!(Warning, &error, "Just a string literal");

    // Severity, literal
    log!(Warning, "Literal with no error");

    // Logger, severity, error, literal, format args
    log!(
        &logger,
        Warning,
        &error,
        "Formatting: {}, {}, {}",
        arg1,
        arg2,
        arg3
    );

    // Logger, severity, literal, format args
    log!(
        &logger,
        Warning,
        "Another formatting: {}, {}, {}",
        arg1,
        arg2,
        arg3
    );

    // Logger, severity, error, literal
    log!(&logger, Warning, &error, "Just a string literal");

    // Logger, severity, literal
    log!(&logger, Warning, "Literal with no error");
}
