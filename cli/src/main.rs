use std::{
    env::{self, VarError},
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

pub(crate) const KILL_TREE_LOG_ENV_KEY: &str = "KILL_TREE_LOG";

pub(crate) enum LogLevel {
    Quiet,
    Info,
    Verbose,
}

impl FromStr for LogLevel {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quiet" => Ok(LogLevel::Quiet),
            "info" => Ok(LogLevel::Info),
            "verbose" => Ok(LogLevel::Verbose),
            _ => Err(format!("Invalid log level: {}", s).into()),
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Quiet => write!(f, "quiet"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Verbose => write!(f, "verbose"),
        }
    }
}

fn parse_log_level_from_env() -> Result<LogLevel, Box<dyn Error>> {
    match env::var(KILL_TREE_LOG_ENV_KEY) {
        Ok(v) => v.parse(),
        Err(VarError::NotPresent) => Ok(LogLevel::Quiet),
        Err(e) => Err(e.into()),
    }
}

fn main() {
    println!("Hello, world!");
}
