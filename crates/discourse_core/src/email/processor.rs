pub struct Processor {
    mail_string: String,
    retry_on_rate_limit: bool,
}

impl Processor {
    pub fn new(mail_string: &str, retry_on_rate_limit: bool) -> Self {
        Self {
            mail_string: mail_string.to_string(),
            retry_on_rate_limit,
        }
    }

    pub fn process(&self) -> Result<(), String> {
        todo!("Orchestrate Receiver to process the incoming mail");
    }
}
