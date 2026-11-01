pub struct MessageIdService;

impl MessageIdService {
    pub fn generate_default() -> String {
        // Needs uuid generation and host lookup
        todo!("Generate a UUID and combine with host_for(Discourse.base_url)");
    }

    pub fn generate_or_use_existing(post_ids: Vec<i32>) -> Vec<String> {
        todo!("ActiveRecord and SQL updates for generating or fetching message ids");
    }

    pub fn find_post_from_message_ids(message_ids: Vec<&str>) -> Option<i32> { // Returning an i32 instead of full Post object for simplicity in stub
        todo!("ActiveRecord lookups across Posts, EmailLog, IncomingEmail");
    }

    pub fn discourse_generated_message_id(message_id: &str) -> bool {
        todo!("Regex match for discourse/post/(\\d+)@host");
    }

    pub fn message_id_rfc_format(message_id: &str) -> String {
        if !message_id.is_empty() && !Self::is_message_id_rfc(message_id) {
            format!("<{}>", message_id)
        } else {
            message_id.to_string()
        }
    }

    pub fn message_id_clean(message_id: &str) -> String {
        if !message_id.is_empty() && Self::is_message_id_rfc(message_id) {
            message_id.trim_start_matches('<').trim_end_matches('>').to_string()
        } else {
            message_id.to_string()
        }
    }

    pub fn is_message_id_rfc(message_id: &str) -> bool {
        message_id.starts_with('<') && message_id.contains('@') && message_id.ends_with('>')
    }

    pub fn host() -> String {
        todo!("Call Email::Sender::host_for(Discourse.base_url)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_message_id_rfc() {
        assert!(MessageIdService::is_message_id_rfc("<test@example.com>"));
        assert!(!MessageIdService::is_message_id_rfc("test@example.com"));
    }

    #[test]
    fn test_message_id_rfc_format() {
        assert_eq!(MessageIdService::message_id_rfc_format("test@example.com"), "<test@example.com>");
        assert_eq!(MessageIdService::message_id_rfc_format("<test@example.com>"), "<test@example.com>");
        assert_eq!(MessageIdService::message_id_rfc_format(""), "");
    }

    #[test]
    fn test_message_id_clean() {
        assert_eq!(MessageIdService::message_id_clean("<test@example.com>"), "test@example.com");
        assert_eq!(MessageIdService::message_id_clean("test@example.com"), "test@example.com");
    }
}
