#[derive(Clone, Debug)]
pub struct DevLogFormatter {
    pub color: bool,
}

impl DevLogFormatter {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    pub fn colored(&self) -> Self {
        let mut cloned = self.clone();
        cloned.color = true;
        cloned
    }

    pub fn format(&self, severity: &str, msg: &str) -> String {
        let reset = "\x1b[0m";

        if self.color {
            let code = match severity {
                "DEBUG" => "\x1b[90m",
                "WARN" => "\x1b[33m",
                "ERROR" => "\x1b[31m",
                "FATAL" => "\x1b[1;31m",
                "UNKNOWN" => "\x1b[35m",
                _ => "",
            };

            if code.is_empty() {
                format!("{}\n", msg)
            } else {
                format!("{}{}{}\n", code, msg, reset)
            }
        } else {
            format!("{:<5}  {}\n", severity, msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncolored_format() {
        let formatter = DevLogFormatter::new(false);
        assert_eq!(
            formatter.format("INFO", "test message"),
            "INFO   test message\n"
        );
        assert_eq!(formatter.format("ERROR", "error msg"), "ERROR  error msg\n");
    }

    #[test]
    fn test_colored_format() {
        let formatter = DevLogFormatter::new(true);
        assert_eq!(
            formatter.format("ERROR", "test message"),
            "\x1b[31mtest message\x1b[0m\n"
        );
        assert_eq!(formatter.format("INFO", "info msg"), "info msg\n");
    }

    #[test]
    fn test_colored_method() {
        let formatter = DevLogFormatter::new(false);
        let colored_formatter = formatter.colored();
        assert!(!formatter.color);
        assert!(colored_formatter.color);
    }
}
