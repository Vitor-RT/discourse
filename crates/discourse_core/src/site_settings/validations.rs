pub const PROHIBITED_USER_AGENT_STRINGS: &[&str] = &[
    "apple",
    "windows",
    "linux",
    "ubuntu",
    "gecko",
    "firefox",
    "chrome",
    "safari",
    "applewebkit",
    "webkit",
    "mozilla",
    "macintosh",
    "khtml",
    "intel",
    "osx",
    "os x",
    "iphone",
    "ipad",
    "mac",
];

pub fn validate_cors_origins(new_val: &str) -> Result<(), &'static str> {
    if new_val.is_empty() {
        return Ok(());
    }

    if new_val.split('|').any(|origin| origin.ends_with('/')) {
        return Err("cors_origins_should_not_have_trailing_slash");
    }

    Ok(())
}

pub fn validate_slow_down_crawler_user_agents(new_val: &str) -> Result<(), &'static str> {
    if new_val.is_empty() {
        return Ok(());
    }

    let lower_val = new_val.to_lowercase();
    let agents: Vec<&str> = lower_val.split('|').collect();

    for crawler in agents {
        if crawler.len() < 3 {
            return Err("slow_down_crawler_user_agent_must_be_at_least_3_characters");
        }

        if PROHIBITED_USER_AGENT_STRINGS
            .iter()
            .any(|c| c.contains(crawler))
        {
            return Err("slow_down_crawler_user_agent_cannot_be_popular_browsers");
        }
    }

    Ok(())
}

pub fn split_s3_bucket(s3_bucket: &str) -> (String, Option<String>) {
    let lower = s3_bucket.to_lowercase();
    let mut parts = lower.splitn(2, '/');
    let bucket_name = parts.next().unwrap_or("").to_string();
    let prefix = parts.next().map(|p| p.trim_end_matches('/').to_string());
    (bucket_name, prefix)
}

pub fn validate_bucket_setting(
    _setting_name: &str,
    upload_bucket: &str,
    backup_bucket: &str,
) -> Result<(), &'static str> {
    if upload_bucket.is_empty() || backup_bucket.is_empty() {
        return Ok(());
    }

    let (backup_bucket_name, backup_prefix) = split_s3_bucket(backup_bucket);
    let (upload_bucket_name, upload_prefix) = split_s3_bucket(upload_bucket);

    if backup_bucket_name != upload_bucket_name {
        return Ok(());
    }

    let has_conflict = match (&backup_prefix, &upload_prefix) {
        (None, _) => true, // if backup prefix is empty, it covers everything
        (Some(b_pref), Some(u_pref)) => u_pref.starts_with(b_pref),
        (Some(_), None) => false, // uploading to root while backup is in folder is okay
    };

    if has_conflict {
        return Err("s3_bucket_reused");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cors_origins() {
        assert!(validate_cors_origins("").is_ok());
        assert!(validate_cors_origins("http://example.com|http://test.com").is_ok());
        assert!(validate_cors_origins("http://example.com/").is_err());
        assert!(validate_cors_origins("http://example.com|http://test.com/").is_err());
    }

    #[test]
    fn test_validate_slow_down_crawler_user_agents() {
        assert!(validate_slow_down_crawler_user_agents("").is_ok());
        assert!(validate_slow_down_crawler_user_agents("bot|crawler").is_ok());
        assert!(validate_slow_down_crawler_user_agents("ab").is_err());
        assert!(validate_slow_down_crawler_user_agents("bot|ab").is_err());
        assert!(validate_slow_down_crawler_user_agents("firefox").is_err());
        assert!(validate_slow_down_crawler_user_agents("mozilla").is_err());
    }

    #[test]
    fn test_split_s3_bucket() {
        assert_eq!(
            split_s3_bucket("my-bucket"),
            ("my-bucket".to_string(), None)
        );
        assert_eq!(
            split_s3_bucket("my-bucket/prefix"),
            ("my-bucket".to_string(), Some("prefix".to_string()))
        );
        assert_eq!(
            split_s3_bucket("my-bucket/prefix/nested/"),
            ("my-bucket".to_string(), Some("prefix/nested".to_string()))
        );
    }

    #[test]
    fn test_validate_bucket_setting() {
        assert!(validate_bucket_setting("test", "bucket1", "bucket2").is_ok());
        assert!(validate_bucket_setting("test", "bucket1/upload", "bucket1/backup").is_ok());
        assert!(validate_bucket_setting("test", "bucket1", "bucket1").is_err());
        assert!(validate_bucket_setting("test", "bucket1/uploads", "bucket1").is_err());
        assert!(
            validate_bucket_setting("test", "bucket1/backup/nested", "bucket1/backup").is_err()
        );
    }
}
