use super::onp_diff::{ONPDiff, Operation};
use html_escape;
use regex::Regex;

pub struct DiscourseDiff {
    pub before: String,
    pub after: String,
}

impl DiscourseDiff {
    pub fn new(before: String, after: String) -> Self {
        Self { before, after }
    }

    pub fn inline_html(&self) -> String {
        let block_diff = self.block_by_block_diff();
        let mut inline = Vec::new();

        let mut i = 0;
        while i < block_diff.len() {
            let op_code = block_diff[i].1;

            if op_code == Operation::Common {
                inline.push(block_diff[i].0.clone());
            } else {
                let opposite_op_code = if op_code == Operation::Add {
                    Operation::Delete
                } else {
                    Operation::Add
                };

                let klass = if op_code == Operation::Delete {
                    "diff-del"
                } else {
                    "diff-ins"
                };

                let mut first = i;
                let mut second = i + 1;
                if op_code == Operation::Add {
                    first = i + 1;
                    second = i;
                }

                if i + 1 < block_diff.len() && block_diff[i + 1].1 == opposite_op_code {
                    let before_tokens = self.tokenize_html(&block_diff[first].0);
                    let after_tokens = self.tokenize_html(&block_diff[second].0);

                    let before_refs: Vec<&str> = before_tokens.iter().map(|s| s.as_str()).collect();
                    let after_refs: Vec<&str> = after_tokens.iter().map(|s| s.as_str()).collect();

                    let mut onp = ONPDiff::new(&before_refs, &after_refs, 100_000);
                    let diff = onp.diff().unwrap_or_default();
                    inline.push(self.generate_inline_html(&diff));
                    i += 1;
                } else {
                    inline.push(self.add_class_or_wrap_in_tags(&block_diff[i].0, klass));
                }
            }
            i += 1;
        }

        inline.join("")
    }

    pub fn side_by_side_html(&self) -> String {
        let block_diff = self.block_by_block_diff();
        let mut left = Vec::new();
        let mut right = Vec::new();

        let mut i = 0;
        while i < block_diff.len() {
            let op_code = block_diff[i].1;

            if op_code == Operation::Common {
                left.push(block_diff[i].0.clone());
                right.push(block_diff[i].0.clone());
            } else {
                let opposite_op_code = if op_code == Operation::Add {
                    Operation::Delete
                } else {
                    Operation::Add
                };

                let klass = if op_code == Operation::Delete {
                    "diff-del"
                } else {
                    "diff-ins"
                };

                let mut first = i;
                let mut second = i + 1;
                if op_code == Operation::Add {
                    first = i + 1;
                    second = i;
                }

                if i + 1 < block_diff.len() && block_diff[i + 1].1 == opposite_op_code {
                    let before_tokens = self.tokenize_html(&block_diff[first].0);
                    let after_tokens = self.tokenize_html(&block_diff[second].0);

                    let before_refs: Vec<&str> = before_tokens.iter().map(|s| s.as_str()).collect();
                    let after_refs: Vec<&str> = after_tokens.iter().map(|s| s.as_str()).collect();

                    let mut onp = ONPDiff::new(&before_refs, &after_refs, 100_000);
                    let diff = onp.diff().unwrap_or_default();

                    let (deleted, inserted) = self.generate_side_by_side_html(&diff);
                    left.push(deleted);
                    right.push(inserted);
                    i += 1;
                } else {
                    let wrapped = self.add_class_or_wrap_in_tags(&block_diff[i].0, klass);
                    if op_code == Operation::Delete {
                        left.push(wrapped);
                    } else {
                        right.push(wrapped);
                    }
                }
            }
            i += 1;
        }

        format!(
            "<div class=\"rows\"><div class=\"row\"><div class=\"as-table\"><div class=\"cell del\">{}</div><div class=\"cell ins\">{}</div></div></div></div>",
            left.join(""),
            right.join("")
        )
    }

    pub fn side_by_side_markdown(&self) -> String {
        let line_diff = self.line_by_line_diff();
        let mut table = vec!["<table class=\"diff-side-by-side\">".to_string()];

        let mut i = 0;
        while i < line_diff.len() {
            let op_code = line_diff[i].1;

            table.push("<tr>".to_string());

            if op_code == Operation::Common {
                table.push(format!("<td class=\"--previous\">{}</td>", line_diff[i].0));
                table.push(format!("<td class=\"--current\">{}</td>", line_diff[i].0));
            } else {
                let opposite_op_code = if op_code == Operation::Add {
                    Operation::Delete
                } else {
                    Operation::Add
                };

                let mut first = i;
                let mut second = i + 1;
                if op_code == Operation::Add {
                    first = i + 1;
                    second = i;
                }

                if i + 1 < line_diff.len() && line_diff[i + 1].1 == opposite_op_code {
                    let before_tokens: Vec<String>;
                    let after_tokens: Vec<String>;

                    if line_diff[first].0.lines().count() > 1
                        || line_diff[second].0.lines().count() > 1
                    {
                        before_tokens = vec![line_diff[first].0.clone()];
                        after_tokens = vec![line_diff[second].0.clone()];
                    } else {
                        before_tokens = self.tokenize_line(&line_diff[first].0);
                        after_tokens = self.tokenize_line(&line_diff[second].0);
                    }

                    let before_refs: Vec<&str> = before_tokens.iter().map(|s| s.as_str()).collect();
                    let after_refs: Vec<&str> = after_tokens.iter().map(|s| s.as_str()).collect();

                    let mut onp = ONPDiff::new(&before_refs, &after_refs, 100_000);
                    let diff = onp.short_diff().unwrap_or_default();

                    let (deleted, inserted) = self.generate_side_by_side_markdown(&diff);
                    table.push(format!(
                        "<td class=\"--previous\">{}</td>",
                        deleted.join("")
                    ));
                    table.push(format!(
                        "<td class=\"--current\">{}</td>",
                        inserted.join("")
                    ));
                    i += 1;
                } else {
                    if op_code == Operation::Delete {
                        table.push(format!(
                            "<td class=\"--previous diff-del\">{}</td>",
                            line_diff[i].0
                        ));
                        table.push("<td class=\"--current\"></td>".to_string());
                    } else {
                        table.push("<td class=\"--previous\"></td>".to_string());
                        table.push(format!(
                            "<td class=\"--current diff-ins\">{}</td>",
                            line_diff[i].0
                        ));
                    }
                }
            }

            table.push("</tr>".to_string());
            i += 1;
        }

        table.push("</table>".to_string());
        table.join("")
    }

    fn block_by_block_diff(&self) -> Vec<(String, Operation)> {
        let before_html = self.tokenize_html_blocks(&self.before);
        let after_html = self.tokenize_html_blocks(&self.after);

        let before_refs: Vec<&str> = before_html.iter().map(|s| s.as_str()).collect();
        let after_refs: Vec<&str> = after_html.iter().map(|s| s.as_str()).collect();

        let mut onp = ONPDiff::new(&before_refs, &after_refs, 100_000);
        onp.paragraph_diff().unwrap_or_default()
    }

    fn line_by_line_diff(&self) -> Vec<(String, Operation)> {
        let before_markdown = self.tokenize_line(&html_escape::encode_text(&self.before));
        let after_markdown = self.tokenize_line(&html_escape::encode_text(&self.after));

        let before_refs: Vec<&str> = before_markdown.iter().map(|s| s.as_str()).collect();
        let after_refs: Vec<&str> = after_markdown.iter().map(|s| s.as_str()).collect();

        let mut onp = ONPDiff::new(&before_refs, &after_refs, 100_000);
        onp.short_diff().unwrap_or_default()
    }

    fn tokenize_line(&self, text: &str) -> Vec<String> {
        let re = Regex::new(r"[^\r\n]+[\r\n]*").unwrap();
        re.find_iter(text).map(|m| m.as_str().to_string()).collect()
    }

    fn tokenize_html_blocks(&self, text: &str) -> Vec<String> {
        let re = Regex::new(r"(?s)<[a-zA-Z0-9]+.*?>.*?</[a-zA-Z0-9]+>|<[a-zA-Z0-9]+.*?/>|[^<]+")
            .unwrap();
        re.find_iter(text).map(|m| m.as_str().to_string()).collect()
    }

    fn tokenize_html(&self, text: &str) -> Vec<String> {
        let re = Regex::new(r"<[^>]+>|[^<]+").unwrap();
        re.find_iter(text).map(|m| m.as_str().to_string()).collect()
    }

    fn generate_inline_html(&self, diff: &[(String, Operation)]) -> String {
        let mut result = String::new();
        for (text, op) in diff {
            match op {
                Operation::Common => result.push_str(text),
                Operation::Add => result.push_str(&format!("<ins>{}</ins>", text)),
                Operation::Delete => result.push_str(&format!("<del>{}</del>", text)),
            }
        }
        result
    }

    fn generate_side_by_side_html(&self, diff: &[(String, Operation)]) -> (String, String) {
        let mut left = String::new();
        let mut right = String::new();
        for (text, op) in diff {
            match op {
                Operation::Common => {
                    left.push_str(text);
                    right.push_str(text);
                }
                Operation::Delete => left.push_str(&format!("<del>{}</del>", text)),
                Operation::Add => right.push_str(&format!("<ins>{}</ins>", text)),
            }
        }
        (left, right)
    }

    fn generate_side_by_side_markdown(
        &self,
        diff: &[(String, Operation)],
    ) -> (Vec<String>, Vec<String>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for (text, op) in diff {
            match op {
                Operation::Common => {
                    left.push(text.clone());
                    right.push(text.clone());
                }
                Operation::Delete => left.push(format!("<span class=\"diff-del\">{}</span>", text)),
                Operation::Add => right.push(format!("<span class=\"diff-ins\">{}</span>", text)),
            }
        }
        (left, right)
    }

    fn add_class_or_wrap_in_tags(&self, html: &str, klass: &str) -> String {
        if html.starts_with('<') && html.contains('>') {
            let re = Regex::new(r#"^<([a-zA-Z0-9]+)([^>]*)>"#).unwrap();
            if let Some(caps) = re.captures(html) {
                let tag = &caps[1];
                let attrs = &caps[2];
                let rest = &html[caps[0].len()..];
                return format!("<{} class=\"{}\"{}>{}", tag, klass, attrs, rest);
            }
        }
        format!("<div class=\"{}\">{}</div>", klass, html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_html() {
        let diff = DiscourseDiff::new("hello world".to_string(), "hello rust world".to_string());
        let result = diff.inline_html();
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_side_by_side_html() {
        let diff = DiscourseDiff::new("hello world".to_string(), "hello rust world".to_string());
        let result = diff.side_by_side_html();
        assert!(result.contains("class=\"as-table\""));
    }

    #[test]
    fn test_side_by_side_markdown() {
        let diff = DiscourseDiff::new("hello world".to_string(), "hello rust world".to_string());
        let result = diff.side_by_side_markdown();
        assert!(result.contains("diff-side-by-side"));
    }
}
