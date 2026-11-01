use serde_json::{Map, Value};
use std::env;

pub fn enabled() -> bool {
    env::var("ENABLE_LOGSTASH_LOGGER").unwrap_or_else(|_| "".to_string()) == "1"
}

pub fn custom_payload(ip: &str, username: &str, extras: Map<String, Value>) -> Value {
    let mut payload = Map::new();
    payload.insert("ip".to_string(), Value::String(ip.to_string()));
    payload.insert("username".to_string(), Value::String(username.to_string()));

    for (k, v) in extras {
        if !v.is_null() {
            payload.insert(k, v);
        }
    }

    Value::Object(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::env;
<<<<<<< HEAD
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_enabled_true() {
        let _guard = ENV_MUTEX.lock().unwrap();
=======

    #[test]
    fn test_enabled_true() {
>>>>>>> origin/rust-migration-7847495834899956704
        env::set_var("ENABLE_LOGSTASH_LOGGER", "1");
        assert!(enabled());
        env::remove_var("ENABLE_LOGSTASH_LOGGER");
    }

    #[test]
    fn test_enabled_false() {
<<<<<<< HEAD
        let _guard = ENV_MUTEX.lock().unwrap();
=======
>>>>>>> origin/rust-migration-7847495834899956704
        env::set_var("ENABLE_LOGSTASH_LOGGER", "0");
        assert!(!enabled());
        env::remove_var("ENABLE_LOGSTASH_LOGGER");

        assert!(!enabled());
    }

    #[test]
    fn test_custom_payload() {
        let mut extras = Map::new();
        extras.insert("key1".to_string(), json!("value1"));
        extras.insert("key2".to_string(), Value::Null);

        let payload = custom_payload("127.0.0.1", "jules", extras);

        assert_eq!(payload["ip"], "127.0.0.1");
        assert_eq!(payload["username"], "jules");
        assert_eq!(payload["key1"], "value1");
        assert!(payload.get("key2").is_none());
    }
}
