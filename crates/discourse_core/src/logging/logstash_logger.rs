use regex::Regex;
use serde_json::{Map, Value};

type CustomizeEventFn = Box<dyn Fn(&mut Map<String, Value>) + Send + Sync>;

const ALLOWED_HEADERS_FROM_ENV: &[&str] = &[
    "REQUEST_URI",
    "REQUEST_METHOD",
    "HTTP_HOST",
    "HTTP_USER_AGENT",
    "HTTP_ACCEPT",
    "HTTP_REFERER",
    "HTTP_X_FORWARDED_FOR",
];

pub struct DiscourseLogstashLogger {
    pub logdev: Box<dyn std::io::Write + Send + Sync>,
    pub log_type: String,
    pub level: u8,
    pub customize_event: Option<CustomizeEventFn>,
    pub ignore_patterns: Vec<Regex>,
}

impl DiscourseLogstashLogger {
    pub fn new(
        logdev: Box<dyn std::io::Write + Send + Sync>,
        log_type: String,
        level: u8,
        customize_event: Option<CustomizeEventFn>,
    ) -> Self {
        Self {
            logdev,
            log_type,
            level,
            customize_event,
            ignore_patterns: Vec::new(),
        }
    }

    pub fn with_ignore_patterns(mut self, patterns: Vec<Regex>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    pub fn add_with_opts(
        &mut self,
        severity: u8,
        message: Option<&str>,
        progname: Option<&str>,
        opts: Option<&Map<String, Value>>,
    ) -> std::io::Result<()> {
        if severity < self.level {
            return Ok(());
        }

        let msg = message.unwrap_or("").to_string();

        let mut event = Map::new();
        event.insert("message".to_string(), Value::String(msg.clone()));
        event.insert("severity".to_string(), Value::Number(severity.into()));

        let severity_name = match severity {
            0 => "DEBUG",
            1 => "INFO",
            2 => "WARN",
            3 => "ERROR",
            4 => "FATAL",
            _ => "UNKNOWN",
        };
        event.insert(
            "severity_name".to_string(),
            Value::String(severity_name.to_string()),
        );
        event.insert("pid".to_string(), Value::Number(std::process::id().into()));
        event.insert("type".to_string(), Value::String(self.log_type.clone()));

        let host = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());
        event.insert("host".to_string(), Value::String(host));

        // Mock git_version since we're not loading the full ruby app context
        event.insert(
            "git_version".to_string(),
            Value::String("unknown".to_string()),
        );

        if severity >= 2 {
            // WARN and above
            if let Some(opts) = opts {
                if let Some(backtrace) = opts.get("backtrace") {
                    event.insert("backtrace".to_string(), backtrace.clone());
                }

                if let Some(prog) = progname {
                    if prog == "web-exception" {
                        // Check ignore patterns (equivalent to Logster.store.ignore)
                        for pattern in &self.ignore_patterns {
                            if pattern.is_match(&msg) {
                                return Ok(());
                            }
                        }

                        let re = Regex::new(r"^(?P<class>[^\(\)]+)\s{1}\((?P<message>[\s\S]+)\)")
                            .unwrap();
                        if let Some(caps) = re.captures(&msg) {
                            if let (Some(cls), Some(m)) = (caps.name("class"), caps.name("message"))
                            {
                                event.insert(
                                    "exception.class".to_string(),
                                    Value::String(cls.as_str().to_string()),
                                );
                                event.insert(
                                    "exception.message".to_string(),
                                    Value::String(m.as_str().trim().to_string()),
                                );
                            }
                        }

                        if let Some(env) = opts.get("env").and_then(|e| e.as_object()) {
                            for header in ALLOWED_HEADERS_FROM_ENV {
                                if let Some(val) = env.get(*header) {
                                    event.insert(
                                        format!("request.headers.{}", header.to_lowercase()),
                                        val.clone(),
                                    );
                                }
                            }
                        }
                    } else if prog == "sidekiq-exception" {
                        if let Some(ctx) = opts.get("context").and_then(|c| c.as_object()) {
                            if let Some(job) = ctx.get("job") {
                                event.insert("job.class".to_string(), job.clone());
                            }
                            if let Some(job_opts) = ctx.get("opts") {
                                event.insert(
                                    "job.opts".to_string(),
                                    Value::String(job_opts.to_string()),
                                );
                            }
                            if let Some(problem_db) = ctx.get("problem_db") {
                                event.insert("job.problem_db".to_string(), problem_db.clone());
                            }
                        }
                        if let Some(exc_class) = opts.get("exception_class") {
                            event.insert("exception.class".to_string(), exc_class.clone());
                        }
                        if let Some(exc_msg) = opts.get("exception_message") {
                            event.insert("exception.message".to_string(), exc_msg.clone());
                        }
                    }
                }
            }
        }

        if msg.starts_with('{') && msg.ends_with('}') {
            if let Ok(mut parsed) = serde_json::from_str::<Map<String, Value>>(&msg) {
                if let Some(m) = parsed.remove("message") {
                    event.insert("message".to_string(), m);
                }
                for (k, v) in parsed {
                    event.insert(k, v);
                }
            }
        }

        if let Some(customizer) = &self.customize_event {
            customizer(&mut event);
        }

        let json_str = serde_json::to_string(&event).unwrap();
        writeln!(&mut self.logdev, "{}", json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    struct MockWriter {
        data: Arc<Mutex<Vec<u8>>>,
    }

    impl std::io::Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn create_logger() -> (DiscourseLogstashLogger, Arc<Mutex<Vec<u8>>>) {
        let data = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(MockWriter {
            data: Arc::clone(&data),
        });
        let logger = DiscourseLogstashLogger::new(writer, "test".to_string(), 0, None);
        (logger, data)
    }

    #[test]
    fn test_basic_log() {
        let (mut logger, data) = create_logger();
        logger
            .add_with_opts(1, Some("hello world"), None, None)
            .unwrap();

        let output = String::from_utf8(data.lock().unwrap().clone()).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["message"], "hello world");
        assert_eq!(parsed["severity"], 1);
        assert_eq!(parsed["severity_name"], "INFO");
        assert_eq!(parsed["type"], "test");
        assert!(parsed.get("pid").is_some());
    }

    #[test]
    fn test_web_exception() {
        let (mut logger, data) = create_logger();
        let msg = "RuntimeError (Something went wrong)";
        let mut opts = Map::new();
        let mut env = Map::new();
        env.insert("REQUEST_URI".to_string(), json!("/test"));
        opts.insert("env".to_string(), Value::Object(env));

        logger
            .add_with_opts(3, Some(msg), Some("web-exception"), Some(&opts))
            .unwrap();

        let output = String::from_utf8(data.lock().unwrap().clone()).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["exception.class"], "RuntimeError");
        assert_eq!(parsed["exception.message"], "Something went wrong");
        assert_eq!(parsed["request.headers.request_uri"], "/test");
    }

    #[test]
    fn test_web_exception_ignored() {
        let (mut logger, data) = create_logger();
        logger = logger.with_ignore_patterns(vec![Regex::new(r"ignore_me").unwrap()]);

        let msg = "RuntimeError (ignore_me error)";
        let opts = Map::new();

        logger
            .add_with_opts(3, Some(msg), Some("web-exception"), Some(&opts))
            .unwrap();

        let output = String::from_utf8(data.lock().unwrap().clone()).unwrap();
        assert!(output.is_empty(), "Log should have been ignored");
    }

    #[test]
    fn test_sidekiq_exception() {
        let (mut logger, data) = create_logger();
        let mut opts = Map::new();
        let mut context = Map::new();
        context.insert("job".to_string(), json!("MyJob"));
        context.insert("opts".to_string(), json!({"key": "value"}));

        opts.insert("context".to_string(), Value::Object(context));
        opts.insert("exception_class".to_string(), json!("StandardError"));
        opts.insert("exception_message".to_string(), json!("job failed"));

        logger
            .add_with_opts(3, Some("error"), Some("sidekiq-exception"), Some(&opts))
            .unwrap();

        let output = String::from_utf8(data.lock().unwrap().clone()).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["job.class"], "MyJob");
        assert_eq!(parsed["exception.class"], "StandardError");
        assert_eq!(parsed["exception.message"], "job failed");
        assert!(parsed
            .get("job.opts")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("key"));
    }

    #[test]
    fn test_json_message() {
        let (mut logger, data) = create_logger();
        let msg = r#"{"custom_key": "custom_val", "message": "real msg"}"#;

        logger.add_with_opts(1, Some(msg), None, None).unwrap();

        let output = String::from_utf8(data.lock().unwrap().clone()).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["message"], "real msg");
        assert_eq!(parsed["custom_key"], "custom_val");
    }
}
