use std::collections::HashMap;

pub struct Renderer {
    message: String, // Would realistically be a Mail object
    opts: HashMap<String, String>,
    text: Option<String>,
}

impl Renderer {
    pub fn new(message: &str, opts: Option<HashMap<String, String>>) -> Self {
        Self {
            message: message.to_string(),
            opts: opts.unwrap_or_default(),
            text: None,
        }
    }

    pub fn text(&mut self) -> String {
        if let Some(text) = &self.text {
            return text.clone();
        }
        // Need a robust mail parsing crate to do this correctly.
        todo!("Get text_part, force encoding to UTF-8, unescape HTML");
    }

    pub fn html(&self) -> String {
        todo!("Get html_part, apply Email::Styles, handle template rendering and plugins");
    }
}
