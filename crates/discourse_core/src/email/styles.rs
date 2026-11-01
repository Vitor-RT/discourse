pub struct Styles {
    html: String,
}

impl Styles {
    pub fn new(html: &str) -> Self {
        Self {
            html: html.to_string(),
        }
    }

    pub fn format_basic(&mut self) {
        todo!("Apply basic formatting to html");
    }

    pub fn format_html(&mut self) {
        todo!("Apply advanced HTML formatting/CSS inline styles");
    }

    pub fn to_html(&self) -> String {
        self.html.clone()
    }
}
