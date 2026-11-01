pub struct ToS3Migration;

impl ToS3Migration {
    pub fn is_dry_run(args: &[String]) -> bool {
        args.contains(&"--dry-run".to_string()) || args.contains(&"-d".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dry_run() {
        assert!(ToS3Migration::is_dry_run(&["--dry-run".to_string()]));
        assert!(!ToS3Migration::is_dry_run(&["--migrate".to_string()]));
    }
}
