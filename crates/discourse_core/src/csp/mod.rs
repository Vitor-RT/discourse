pub mod builder;
pub mod default;
pub mod extension;
pub mod middleware;

use indexmap::IndexMap;

pub type Directives = IndexMap<String, Vec<String>>;

pub struct ContentSecurityPolicy;

impl ContentSecurityPolicy {
    pub fn nonce_placeholder(nonce_env: Option<&str>, cacheable_env: bool) -> String {
        if nonce_env.is_none() || cacheable_env {
            use std::time::{SystemTime, UNIX_EPOCH};
            // Simplistic random generation for the nonce placeholder to mimic SecureRandom.hex
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!("[[csp_nonce_placeholder_{:x}]]", t)
        } else {
            nonce_env.unwrap().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csp::default::{CspSettings, DefaultPolicy};
    use crate::csp::builder::Builder;

    fn parse_policy(policy: &str) -> Directives {
        let mut parsed = Directives::new();
        for part in policy.split(';') {
            let part = part.trim();
            if part.is_empty() { continue; }
            let mut iter = part.split_whitespace();
            if let Some(directive) = iter.next() {
                let sources: Vec<String> = iter.map(|s| s.to_string()).collect();
                parsed.insert(directive.to_string(), sources);
            }
        }
        parsed
    }

    fn generate_policy(settings: CspSettings) -> String {
        let default_policy = DefaultPolicy::new(&settings);
        let builder = Builder::new(default_policy.directives, settings.base_path.clone());
        builder.build()
    }

    #[test]
    fn test_base_uri_is_self() {
        let policy = generate_policy(CspSettings::default());
        let parsed = parse_policy(&policy);
        assert_eq!(parsed.get("base-uri").unwrap(), &vec!["'self'".to_string()]);
    }

    #[test]
    fn test_object_src_is_none() {
        let policy = generate_policy(CspSettings::default());
        let parsed = parse_policy(&policy);
        assert_eq!(parsed.get("object-src").unwrap(), &vec!["'none'".to_string()]);
    }

    #[test]
    fn test_upgrade_insecure_requests() {
        let mut settings = CspSettings::default();
        settings.force_https = false;
        let policy = generate_policy(settings.clone());
        assert!(parse_policy(&policy).get("upgrade-insecure-requests").is_none());

        settings.force_https = true;
        let policy = generate_policy(settings.clone());
        assert!(parse_policy(&policy).get("upgrade-insecure-requests").is_some());

        settings.report_only = true;
        let policy = generate_policy(settings.clone());
        assert!(parse_policy(&policy).get("upgrade-insecure-requests").is_none());
    }

    #[test]
    fn test_strict_dynamic_script_src_and_worker_src() {
        let policy = generate_policy(CspSettings::default());
        let parsed = parse_policy(&policy);
        let script_src = parsed.get("script-src").unwrap();
        assert!(script_src.contains(&"'strict-dynamic'".to_string()));
        assert!(script_src.contains(&"'wasm-unsafe-eval'".to_string()));

        let worker_src = parsed.get("worker-src").unwrap();
        assert_eq!(worker_src, &vec!["'self'".to_string(), "blob:".to_string()]);
    }

    #[test]
    fn test_worker_src_cdn() {
        let mut settings = CspSettings::default();
        settings.cdn_url = Some("https://cdn.example.com".to_string());
        let policy = generate_policy(settings);
        let parsed = parse_policy(&policy);
        let worker_src = parsed.get("worker-src").unwrap();
        assert_eq!(worker_src, &vec![
            "'self'".to_string(),
            "blob:".to_string(),
            "https://cdn.example.com/assets/".to_string()
        ]);
    }

    #[test]
    fn test_worker_src_s3_cdn() {
        let mut settings = CspSettings::default();
        settings.use_s3 = true;
        settings.s3_cdn_url = Some("https://s3cdn.example.com".to_string());
        let policy = generate_policy(settings);
        let parsed = parse_policy(&policy);
        let worker_src = parsed.get("worker-src").unwrap();
        assert_eq!(worker_src, &vec![
            "'self'".to_string(),
            "blob:".to_string(),
            "https://s3cdn.example.com/assets/".to_string()
        ]);
    }

    #[test]
    fn test_manifest_src() {
        let policy = generate_policy(CspSettings::default());
        let parsed = parse_policy(&policy);
        assert_eq!(parsed.get("manifest-src").unwrap(), &vec!["'self'".to_string()]);
    }

    #[test]
    fn test_report_uri() {
        let mut settings = CspSettings::default();
        settings.report_uri = Some("".to_string());
        let policy = generate_policy(settings.clone());
        let parsed = parse_policy(&policy);
        assert!(parsed.get("report-uri").is_none());
        assert!(!parsed.get("script-src").unwrap().contains(&"'report-sample'".to_string()));

        settings.report_uri = Some("https://csp.example.com/report".to_string());
        let policy = generate_policy(settings.clone());
        let parsed = parse_policy(&policy);
        assert_eq!(parsed.get("report-uri").unwrap(), &vec!["https://csp.example.com/report".to_string()]);
        assert!(parsed.get("script-src").unwrap().contains(&"'report-sample'".to_string()));
    }

    #[test]
    fn test_frame_ancestors_enabled() {
        let mut settings = CspSettings::default();
        settings.frame_ancestors = true;
        settings.embeddable_hosts = vec!["a.org".to_string(), "b.org".to_string()];
        let policy = generate_policy(settings);
        let parsed = parse_policy(&policy);
        let ancestors = parsed.get("frame-ancestors").unwrap();
        assert!(ancestors.contains(&"'self'".to_string()));
        assert!(ancestors.contains(&"https://a.org".to_string()));
        assert!(ancestors.contains(&"https://b.org".to_string()));
    }

    #[test]
    fn test_frame_ancestors_disabled() {
        let mut settings = CspSettings::default();
        settings.frame_ancestors = false;
        let policy = generate_policy(settings);
        let parsed = parse_policy(&policy);
        assert!(parsed.get("frame-ancestors").is_none());
    }

    #[test]
    fn test_builder_extension() {
        let default_policy = DefaultPolicy::new(&CspSettings::default());
        let mut builder = Builder::new(default_policy.directives, "".to_string());

        let mut extension = Directives::new();
        extension.insert("script_src".to_string(), vec!["'unsafe-eval'".to_string()]);
        builder.extend(&extension);

        let policy = builder.build();
        let parsed = parse_policy(&policy);
        assert!(parsed.get("script-src").unwrap().contains(&"'unsafe-eval'".to_string()));
    }
}
