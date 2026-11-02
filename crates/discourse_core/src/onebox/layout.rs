use super::view::View;
use std::collections::HashMap;
use url::Url;

pub struct Layout {
    pub record: HashMap<String, String>,
    pub view: View,
}

impl Layout {
    pub fn new(name: &str, mut record: HashMap<String, String>) -> Self {
        // Fix any relative paths
        if let Some(image) = record.get("image").cloned() {
            if image.starts_with('/') && !image.starts_with("//") {
                if let Some(link) = record.get("link") {
                    if let Ok(uri) = Url::parse(link) {
                        record.insert(
                            "image".to_string(),
                            format!(
                                "{}://{}{}",
                                uri.scheme(),
                                uri.host_str().unwrap_or(""),
                                image
                            ),
                        );
                    }
                }
            }
        }

        Self {
            view: View::new(name, record.clone()),
            record,
        }
    }

    pub fn to_html(&self) -> String {
        // details rendering stub
        "".to_string()
    }
}
