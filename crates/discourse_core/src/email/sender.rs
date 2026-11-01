pub struct Sender {
    message: String,
    email_type: String,
    user_id: Option<i32>,
}

impl Sender {
    pub fn new(message: &str, email_type: &str, user_id: Option<i32>) -> Self {
        Self {
            message: message.to_string(),
            email_type: email_type.to_string(),
            user_id,
        }
    }

    pub fn send(&self) -> Result<(), String> {
        todo!("Send email using Mail crate or SMTP library");
    }

    pub fn host_for(base_url: &str) -> String {
        todo!("Determine host from base_url or site settings");
    }
}
