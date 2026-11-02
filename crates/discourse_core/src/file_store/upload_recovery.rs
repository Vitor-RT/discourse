pub struct UploadRecovery {
    pub dry_run: bool,
    pub stop_on_error: bool,
}

impl UploadRecovery {
    pub fn new(dry_run: bool, stop_on_error: bool) -> Self {
        Self {
            dry_run,
            stop_on_error,
        }
    }

    pub fn valid_sha1(sha1: &str) -> bool {
        !sha1.is_empty() && sha1.len() == 40 && sha1.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sha1() {
        assert!(UploadRecovery::valid_sha1(
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        ));
        assert!(!UploadRecovery::valid_sha1("short"));
        assert!(!UploadRecovery::valid_sha1(""));
    }
}
