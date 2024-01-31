pub const KILL_TREE_LOG_ENV_KEY: &str = "KILL_TREE_LOG";

// log level quiet, info, verbose
// library default log level is quiet
// cli default log level is info
// cli --log-level <level> is log level <level>
// cli -v --verbose is equivalent to --log-level verbose
// cli -q --quiet is equivalent to --log-level quiet
// cli --log-level and --verbose and --quiet are mutually exclusive
// env var KILL_TREE_LOG is higher priority than cli --log-level

enum LogLevel {
    Quiet,
    Info,
    Verbose,
}

impl FromStr for LogLevel {
    fn from_str(s: &str) -> Result<Self, std::error::Error> {
        match s.to_lowercase().as_str() {
            "quiet" => Ok(LogLevel::Quiet),
            "info" => Ok(LogLevel::Info),
            "verbose" => Ok(LogLevel::Verbose),
            _ => Err(format!("Invalid log level: {}", s).into()),
        }
    }
}

fn parse_log_level_from_env() -> Result<LogLevel, Box<dyn std::error::Error>> {
    match env::var(KILL_TREE_LOG_ENV_KEY) {
        Ok(v) => Ok(v.parse()?),
        Err(env::VarError::NotPresent) => Ok(LogLevel::Quiet),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_log_level_from_env() {
        env::set_var(KILL_TREE_LOG_ENV_KEY, "quiet");
        assert_eq!(parse_log_level_from_env().unwrap(), LogLevel::Quiet);
        env::set_var(KILL_TREE_LOG_ENV_KEY, "info");
        assert_eq!(parse_log_level_from_env().unwrap(), LogLevel::Info);
        env::set_var(KILL_TREE_LOG_ENV_KEY, "verbose");
        assert_eq!(parse_log_level_from_env().unwrap(), LogLevel::Verbose);
        env::set_var(KILL_TREE_LOG_ENV_KEY, "invalid");
        assert!(parse_log_level_from_env().is_err());
    }
}
