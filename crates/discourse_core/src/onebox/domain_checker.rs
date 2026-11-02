pub struct DomainChecker;

impl DomainChecker {
    pub fn is_blocked(hostname: &str, blocked_domains: &[&str]) -> bool {
        let normalized_hostname = hostname.to_lowercase();
        blocked_domains.iter().any(|blocked| {
            let normalized_blocked = blocked.to_lowercase();
            normalized_hostname == normalized_blocked
                || normalized_hostname.ends_with(&format!(".{}", normalized_blocked))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_blocked() {
        let blocked = vec!["example.com"];
        assert!(DomainChecker::is_blocked("example.com", &blocked));
        assert!(DomainChecker::is_blocked("sub.example.com", &blocked));
        assert!(!DomainChecker::is_blocked("example.org", &blocked));
        assert!(!DomainChecker::is_blocked("myexample.com", &blocked));
    }
}
