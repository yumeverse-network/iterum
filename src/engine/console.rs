use chrono::Local;
use std::fmt;

struct Timestamp;

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Local::now().format("%H:%M:%S:%3f"))
    }
}

pub struct Console;

impl Console {
    pub fn new() -> Self {
        Console {}
    }

    pub fn log<T: std::fmt::Display>(&self, msg: T) {
        println!("\x1b[37m{} Info: {}\x1b[0m", Timestamp, msg);
    }

    pub fn log_system<T: std::fmt::Display>(&self, msg: T) {
        println!("\x1b[37m{} System: {}\x1b[0m", Timestamp, msg);
    }

    pub fn log_debug<T: std::fmt::Display>(&self, msg: T) {
        println!("\x1b[33m{} Debug: {}\x1b[0m", Timestamp, msg);
    }

    pub fn log_error<T: std::fmt::Display>(&self, msg: T) {
        println!("\x1b[31m{} Error: {}\x1b[0m", Timestamp, msg);
    }

    pub fn log_warning<T: std::fmt::Display>(&self, msg: T) {
        println!("\x1b[33m{} Warning: {}\x1b[0m", Timestamp, msg);
    }

    pub fn break_line(&self) {
        println!();
    }
}
