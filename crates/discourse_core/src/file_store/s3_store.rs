pub const CANNED_ACL_PUBLIC_READ: &str = "public-read";
pub const CANNED_ACL_PRIVATE: &str = "private";
pub const MAX_CONTENT_DISPOSITION_BYTES: usize = 1600;

pub struct S3Store;

impl S3Store {
    pub fn acl_option_value(secure: bool, s3_use_acls: bool) -> Option<&'static str> {
        if !s3_use_acls {
            return None;
        }
        if secure {
            Some(CANNED_ACL_PRIVATE)
        } else {
            Some(CANNED_ACL_PUBLIC_READ)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_store_acl() {
        assert_eq!(S3Store::acl_option_value(true, true), Some("private"));
        assert_eq!(S3Store::acl_option_value(false, true), Some("public-read"));
        assert_eq!(S3Store::acl_option_value(true, false), None);
    }
}
