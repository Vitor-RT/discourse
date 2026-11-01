pub struct Receiver {
    raw: String,
}

impl Receiver {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub fn process(&self) -> Result<(), String> {
        todo!("Process incoming email, interact with IncomingEmail, Posts, Users, etc.");
    }
}
