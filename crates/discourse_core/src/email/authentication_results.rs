pub struct AuthenticationResults {
    verdicts: std::collections::HashMap<String, String>,
}

impl AuthenticationResults {
    pub fn new(headers: &str) -> Self {
        todo!("Parse Authentication-Results headers");
    }

    pub fn dkim_pass(&self) -> bool {
        self.verdicts.get("dkim").map(|v| v == "pass").unwrap_or(false)
    }

    pub fn spf_pass(&self) -> bool {
        self.verdicts.get("spf").map(|v| v == "pass").unwrap_or(false)
    }

    pub fn dmarc_pass(&self) -> bool {
        self.verdicts.get("dmarc").map(|v| v == "pass").unwrap_or(false)
    }
}
