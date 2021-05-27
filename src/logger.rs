#![allow(dead_code)]

use console::Style;
pub enum Logger {
    Info,
    Warn,
    Error,
}

impl Logger {
    fn log(&self, msg: String) {
        match self {
            Logger::Info => {
                println!(
                    "{}: {}",
                    Style::new().bold().apply_to("INFO"),
                    Style::new().dim().apply_to(&msg)
                );
            }
            Logger::Warn => {
                eprintln!(
                    "{}: {}",
                    Style::new().yellow().bold().apply_to("WARN"),
                    Style::new().dim().apply_to(&msg)
                );
            }
            Logger::Error => {
                eprintln!(
                    "{}: {}",
                    Style::new().red().bold().apply_to("ERROR"),
                    Style::new().dim().apply_to(&msg)
                );
            }
        }
    }

    pub fn error(msg: String) {
        Self::log(&Logger::Error, msg);
    }

    pub fn warn(msg: String) {
        Self::log(&Logger::Warn, msg);
    }

    pub fn info(msg: String) {
        Self::log(&Logger::Info, msg);
    }
}
