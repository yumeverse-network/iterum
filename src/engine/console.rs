use chrono::Local;
use std::fmt;
use std::sync::{Mutex, OnceLock};

pub static CONSOLE_LINES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn console_lines() -> &'static Mutex<Vec<String>> {
    CONSOLE_LINES.get_or_init(|| Mutex::new(Vec::new()))
}

const SKIP: &[&str] = &["wgpu", "polling", "surface configure"];

struct Timestamp;

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Local::now().format("%H:%M:%S:%3f"))
    }
}

pub struct Console;

impl Console {
    pub fn new() -> Self { Console {} }

    fn push(&self, color: &str, prefix: &str, msg: &str) {
        let line = format!("{} {}: {}", Timestamp, prefix, msg);
        if SKIP.iter().any(|s| line.contains(s)) { return; }
        println!("\x1b[{}m{}\x1b[0m", color, line);
        let mut v = console_lines().lock().unwrap();
        v.push(line);
        if v.len() > 5000 { v.remove(0); }
    }

    pub fn log<T: fmt::Display>(&self, msg: T)         { self.push("37", "Info", &msg.to_string()); }
    pub fn log_system<T: fmt::Display>(&self, msg: T)  { self.push("37", "System", &msg.to_string()); }
    pub fn log_debug<T: fmt::Display>(&self, msg: T)   { self.push("33", "Debug", &msg.to_string()); }
    pub fn log_error<T: fmt::Display>(&self, msg: T)   { self.push("31", "Error", &msg.to_string()); }
    pub fn log_warning<T: fmt::Display>(&self, msg: T) { self.push("33", "Warning", &msg.to_string()); }

    pub fn break_line(&self) {
        println!();
    }
}