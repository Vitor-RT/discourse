pub struct View {
    pub name: String,
    pub record: std::collections::HashMap<String, String>,
}

impl View {
    pub fn new(name: &str, record: std::collections::HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            record,
        }
    }

    pub fn to_html(&self) -> String {
        // mustache render stub
        "".to_string()
    }
}
