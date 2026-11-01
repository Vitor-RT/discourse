pub struct Cleaner {
    mail: String,
    remove_attachments: bool,
    truncate: bool,
    rejected: bool,
}

impl Cleaner {
    pub fn new(mail: &str, remove_attachments: bool, truncate: bool, rejected: bool) -> Self {
        Self {
            mail: mail.to_string(),
            remove_attachments,
            truncate,
            rejected,
        }
    }

    pub fn execute(&mut self) -> String {
        if self.remove_attachments {
            self.remove_attachments();
        }
        if self.truncate {
            self.truncate();
        }
        self.remove_null_byte();
        self.mail.clone()
    }

    pub fn delete_rejected() {
        todo!("ActiveRecord IncomingEmail.delete_by(...)");
    }

    fn remove_attachments(&mut self) {
        // Requires MIME parsing
        todo!("Remove attachments from email string");
    }

    fn truncate(&mut self) {
        // Requires MIME parsing to get parts
        todo!("Truncate email parts");
    }

    fn remove_null_byte(&mut self) {
        self.mail = self.mail.replace('\x00', "");
    }
}
