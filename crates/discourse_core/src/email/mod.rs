pub mod authentication_results;
pub mod build_email_helper;
pub mod cleaner;
pub mod message_builder;
pub mod message_id_service;
pub mod poller;
pub mod processor;
pub mod receiver;
pub mod renderer;
pub mod sender;
pub mod sns;
pub mod styles;
pub mod validator;

pub const SMTP_STATUS_SUCCESS: &str = "2.";
pub const SMTP_STATUS_TRANSIENT_FAILURE: &str = "4.";
pub const SMTP_STATUS_PERMANENT_FAILURE: &str = "5.";

pub fn is_valid(email: &str) -> bool {
    validator::valid_value(email)
}

pub fn downcase(email: &str) -> String {
    if !is_valid(email) {
        return email.to_string();
    }
    email.to_lowercase()
}

pub fn obfuscate(email: &str) -> String {
    if !is_valid(email) {
        return email.to_string();
    }

    let parts: Vec<&str> = email.rsplitn(2, '@').collect();
    if parts.len() != 2 {
        return email.to_string();
    }

    let last = parts[0];
    let first = parts[1];

    let mut last_parts: Vec<&str> = last.split('.').collect();
    let tld = if !last_parts.is_empty() {
        last_parts.pop().unwrap()
    } else {
        ""
    };

    let obfuscated_last: Vec<String> = last_parts.iter().map(|p| obfuscate_part(p)).collect();

    let mut obfuscated_last_joined = obfuscated_last.join(".");
    if !obfuscated_last_joined.is_empty() && !tld.is_empty() {
        obfuscated_last_joined.push('.');
        obfuscated_last_joined.push_str(tld);
    } else if !tld.is_empty() {
        obfuscated_last_joined.push_str(tld);
    }

    format!("{}@{}", obfuscate_part(first), obfuscated_last_joined)
}

fn obfuscate_part(part: &str) -> String {
    let chars: Vec<char> = part.chars().collect();
    let size = chars.len();
    if size < 3 {
        "*".repeat(size)
    } else if size < 5 {
        format!("{}{}", chars[0], "*".repeat(size - 1))
    } else {
        format!("{}{}{}", chars[0], "*".repeat(size - 2), chars[size - 1])
    }
}

pub fn cleanup_alias(name: Option<&str>) -> Option<String> {
    name.map(|n| {
        let mut s = String::new();
        for c in n.chars() {
            if !":<>,\\\"".contains(c) {
                s.push(c);
            }
        }
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downcase() {
        assert_eq!(downcase("A@B.COM"), "a@b.com");
        assert_eq!(downcase("invalid"), "invalid");
    }

    #[test]
    fn test_obfuscate() {
        assert_eq!(obfuscate("test@example.com"), "t***@e*****e.com");
        assert_eq!(obfuscate("t@example.com"), "*@e*****e.com");
        assert_eq!(obfuscate("test@ex.com"), "t***@**.com");
        assert_eq!(obfuscate("invalid"), "invalid");
    }

    #[test]
    fn test_cleanup_alias() {
        assert_eq!(
            cleanup_alias(Some("Alice <alice@example.com>")),
            Some("Alice alice@example.com".to_string())
        );
        assert_eq!(cleanup_alias(Some("\"Alice\"")), Some("Alice".to_string()));
        assert_eq!(cleanup_alias(None), None);
    }
}
