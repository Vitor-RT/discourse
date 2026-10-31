use super::onp_diff::{ONPDiff, Operation};
use html_escape;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Node};

const MAX_DIFFERENCE: isize = 200;
const CLASS_ATTRIBUTE: &str = " class=\"";

lazy_static! {
    static ref WORD_REGEX: Regex = Regex::new(r"\W|\w+[ \t]*").unwrap();
}

pub struct DiscourseDiff {
    before: String,
    after: String,
}

impl DiscourseDiff {
    pub fn new(before: &str, after: &str) -> Self {
        Self {
            before: before.to_string(),
            after: after.to_string(),
        }
    }

    pub fn inline_html(&self) -> String {
        let block_diff = self.block_by_block_diff();
        let mut i = 0;
        let mut inline = Vec::new();

        while i < block_diff.len() {
            let op_code = block_diff[i].1;
            if op_code == Operation::Common {
                inline.push(block_diff[i].0.clone());
            } else {
                let opposite_op_code;
                let klass;
                let first;
                let second;

                if op_code == Operation::Delete {
                    opposite_op_code = Operation::Add;
                    klass = "del";
                    first = i;
                    second = i + 1;
                } else {
                    opposite_op_code = Operation::Delete;
                    klass = "ins";
                    first = i + 1;
                    second = i;
                }

                if i + 1 < block_diff.len() && block_diff[i + 1].1 == opposite_op_code {
                    let before_tokens = self.tokenize_html(&block_diff[first].0);
                    let after_tokens = self.tokenize_html(&block_diff[second].0);

                    let before_refs: Vec<&str> = before_tokens.iter().map(|s| s.as_str()).collect();
                    let after_refs: Vec<&str> = after_tokens.iter().map(|s| s.as_str()).collect();

                    let mut onp = ONPDiff::new(before_refs, after_refs);
                    let diff = onp.diff().unwrap_or_default();
                    inline.push(self.generate_inline_html(&diff));
                    i += 1;
                } else {
                    inline.push(self.add_class_or_wrap_in_tags(&block_diff[i].0, klass));
                }
            }
            i += 1;
        }

        format!("<div class=\"inline-diff\">{}</div>", inline.join(""))
    }

    pub fn side_by_side_html(&self) -> String {
        let block_diff = self.block_by_block_diff();
        let mut i = 0;
        let mut left = Vec::new();
        let mut right = Vec::new();

        while i < block_diff.len() {
            let op_code = block_diff[i].1;
            if op_code == Operation::Common {
                left.push(block_diff[i].0.clone());
                right.push(block_diff[i].0.clone());
            } else {
                let opposite_op_code;
                let klass;
                let first;
                let second;

                if op_code == Operation::Delete {
                    opposite_op_code = Operation::Add;
                    klass = "del";
                    first = i;
                    second = i + 1;
                } else {
                    opposite_op_code = Operation::Delete;
                    klass = "ins";
                    first = i + 1;
                    second = i;
                }

                if i + 1 < block_diff.len() && block_diff[i + 1].1 == opposite_op_code {
                    let before_tokens = self.tokenize_html(&block_diff[first].0);
                    let after_tokens = self.tokenize_html(&block_diff[second].0);

                    let before_refs: Vec<&str> = before_tokens.iter().map(|s| s.as_str()).collect();
                    let after_refs: Vec<&str> = after_tokens.iter().map(|s| s.as_str()).collect();

                    let mut onp = ONPDiff::new(before_refs, after_refs);
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
            "<div class=\"revision-content --previous\">{}</div><div class=\"revision-content --current\">{}</div>",
            left.join(""),
            right.join("")
        )
    }

    pub fn side_by_side_markdown(&self) -> String {
        let line_diff = self.line_by_line_diff();
        let mut i = 0;
        let mut table = vec!["<table class=\"markdown\">".to_string()];

        while i < line_diff.len() {
            table.push("<tr>".to_string());
            let op_code = line_diff[i].1;

            if op_code == Operation::Common {
                table.push(format!("<td class=\"--previous\">{}</td>", line_diff[i].0));
                table.push(format!("<td class=\"--current\">{}</td>", line_diff[i].0));
            } else {
                let opposite_op_code;
                let first;
                let second;

                if op_code == Operation::Delete {
                    opposite_op_code = Operation::Add;
                    first = i;
                    second = i + 1;
                } else {
                    opposite_op_code = Operation::Delete;
                    first = i + 1;
                    second = i;
                }

                if i + 1 < line_diff.len() && line_diff[i + 1].1 == opposite_op_code {
                    let mut before_tokens = self.tokenize_markdown(&line_diff[first].0);
                    let mut after_tokens = self.tokenize_markdown(&line_diff[second].0);

                    if (before_tokens.len() as isize - after_tokens.len() as isize).abs()
                        > MAX_DIFFERENCE
                    {
                        before_tokens = self.tokenize_line(&line_diff[first].0);
                        after_tokens = self.tokenize_line(&line_diff[second].0);
                    }

                    let before_refs: Vec<&str> = before_tokens.iter().map(|s| s.as_str()).collect();
                    let after_refs: Vec<&str> = after_tokens.iter().map(|s| s.as_str()).collect();

                    let mut onp = ONPDiff::new(before_refs, after_refs);
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

        let mut onp = ONPDiff::new(before_refs, after_refs);
        onp.paragraph_diff().unwrap_or_default()
    }

    fn line_by_line_diff(&self) -> Vec<(String, Operation)> {
        let before_markdown = self.tokenize_line(&html_escape::encode_text(&self.before));
        let after_markdown = self.tokenize_line(&html_escape::encode_text(&self.after));

        let before_refs: Vec<&str> = before_markdown.iter().map(|s| s.as_str()).collect();
        let after_refs: Vec<&str> = after_markdown.iter().map(|s| s.as_str()).collect();

        let mut onp = ONPDiff::new(before_refs, after_refs);
        onp.short_diff().unwrap_or_default()
    }

    fn tokenize_line(&self, text: &str) -> Vec<String> {
        let re = Regex::new(r"[^\r\n]+[\r\n]*").unwrap();
        re.find_iter(text).map(|m| m.as_str().to_string()).collect()
    }

    fn tokenize_markdown(&self, text: &str) -> Vec<String> {
        let mut t = Vec::new();
        let mut tokens: Vec<String> = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if c.is_alphanumeric() || c == '_' {
                t.push(c);
            } else if (c == ' ' || c == '\t')
                && t.iter().all(|&ch| ch.is_alphanumeric() || ch == '_')
                && !t.is_empty()
            {
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                    t.push(chars[i]);
                    i += 1;
                }
                i -= 1;
                tokens.push(t.iter().collect());
                t.clear();
            } else {
                if !t.is_empty() {
                    tokens.push(t.iter().collect());
                    t.clear();
                }
                tokens.push(c.to_string());
            }
            i += 1;
        }

        if !t.is_empty() {
            tokens.push(t.iter().collect());
        }

        tokens
    }

    fn tokenize_html_blocks(&self, html: &str) -> Vec<String> {
        if html.is_empty() {
            return vec![];
        }

        let fragment = Html::parse_fragment(html);
        let mut blocks: Vec<String> = Vec::new();

        for node in fragment.tree.root().children() {
            if let Some(elem) = scraper::ElementRef::wrap(node) {
                blocks.push(elem.html());
            } else if let Node::Text(t) = node.value() {
                let text = t.text.to_string();
                if !text.trim().is_empty() {
                    // Do nothing for text nodes as Nokogiri search "./*" skips them
                }
            }
        }

        blocks
    }

    fn tokenize_html(&self, html: &str) -> Vec<String> {
        HtmlTokenizer::tokenize(html)
    }

    fn add_class_or_wrap_in_tags(&self, html_or_text: &str, klass: &str) -> String {
        if html_or_text.starts_with("</") {
            return html_or_text.to_string();
        }

        let chevron = html_or_text.find('>');
        if !html_or_text.starts_with('<') || chevron.is_none() {
            return format!("<{klass}>{html_or_text}</{klass}>");
        }

        let chevron = chevron.unwrap();
        let mut result = html_or_text.to_string();

        let class_index = result.find(CLASS_ATTRIBUTE);
        if let Some(idx) = class_index {
            if idx <= chevron {
                let insert_pos = idx + CLASS_ATTRIBUTE.len();
                result.insert_str(insert_pos, &format!("diff-{} ", klass));
                return result;
            }
        }

        result.insert_str(chevron, &format!("{}diff-{}\"", CLASS_ATTRIBUTE, klass));
        result
    }

    fn generate_inline_html(&self, diff: &[(String, Operation)]) -> String {
        let mut res = String::new();
        for (text, op_code) in diff {
            if *op_code == Operation::Common {
                res.push_str(text);
            } else {
                let klass = if *op_code == Operation::Delete {
                    "del"
                } else {
                    "ins"
                };
                res.push_str(&self.add_class_or_wrap_in_tags(text, klass));
            }
        }
        res
    }

    fn generate_side_by_side_html(&self, diff: &[(String, Operation)]) -> (String, String) {
        let mut deleted = String::new();
        let mut inserted = String::new();

        for (text, op_code) in diff {
            match op_code {
                Operation::Common => {
                    deleted.push_str(text);
                    inserted.push_str(text);
                }
                Operation::Delete => {
                    deleted.push_str(&self.add_class_or_wrap_in_tags(text, "del"));
                }
                Operation::Add => {
                    inserted.push_str(&self.add_class_or_wrap_in_tags(text, "ins"));
                }
            }
        }

        (deleted, inserted)
    }

    fn generate_side_by_side_markdown(
        &self,
        diff: &[(String, Operation)],
    ) -> (Vec<String>, Vec<String>) {
        let mut deleted = Vec::new();
        let mut inserted = Vec::new();

        for (text, op_code) in diff {
            match op_code {
                Operation::Common => {
                    deleted.push(text.clone());
                    inserted.push(text.clone());
                }
                Operation::Delete => {
                    deleted.push(format!("<del>{}</del>", text));
                }
                Operation::Add => {
                    inserted.push(format!("<ins>{}</ins>", text));
                }
            }
        }

        (deleted, inserted)
    }
}

pub struct HtmlTokenizer;

impl HtmlTokenizer {
    pub fn tokenize(html: &str) -> Vec<String> {
        use html5gum::{Token, Tokenizer};
        let mut t = Vec::new();

        for token in Tokenizer::new(html).infallible() {
            match token {
                Token::StartTag(tag) => {
                    let name = String::from_utf8_lossy(&tag.name).to_string();
                    if name == "html" || name == "body" {
                        continue;
                    }

                    let mut attrs = String::new();
                    for (k, v) in &tag.attributes {
                        let k_str = String::from_utf8_lossy(k);
                        let v_str_cow = String::from_utf8_lossy(v);
                        let v_str = html_escape::encode_double_quoted_attribute(&v_str_cow);
                        attrs.push_str(&format!(" {}=\"{}\"", k_str, v_str));
                    }

                    t.push(format!("<{}{}>", name, attrs));
                }
                Token::EndTag(tag) => {
                    let name = String::from_utf8_lossy(&tag.name).to_string();
                    let autoclosings = vec![
                        "area", "base", "br", "col", "embed", "hr", "img", "input", "meta",
                    ];
                    if name == "html" || name == "body" || autoclosings.contains(&name.as_str()) {
                        continue;
                    }
                    t.push(format!("</{}>", name));
                }
                Token::String(s) => {
                    let string = String::from_utf8_lossy(&s).to_string();
                    if string.trim().is_empty()
                        && t.last().map(|s| s.starts_with('<')).unwrap_or(false)
                    {
                        let last = t.last_mut().unwrap();
                        last.push_str(&string);
                    } else {
                        for mat in WORD_REGEX.find_iter(&string) {
                            t.push(html_escape::encode_text(mat.as_str()).to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_html() {
        let before = "<p>this is a great paragraph</p>";
        let after = "<p>this is a paragraph</p>";
        assert_eq!(
            DiscourseDiff::new(before, after).inline_html(),
            "<div class=\"inline-diff\"><p>this is a <del>great </del>paragraph</p></div>"
        );
    }

    #[test]
    fn test_side_by_side_html() {
        let before = "<p>this is a paragraph</p>";
        let after = "<p>this is a great paragraph</p>";
        assert_eq!(
            DiscourseDiff::new(before, after).side_by_side_html(),
            "<div class=\"revision-content --previous\"><p>this is a paragraph</p></div><div class=\"revision-content --current\"><p>this is a <ins>great </ins>paragraph</p></div>"
        );
    }

    #[test]
    fn test_side_by_side_markdown() {
        let before = "this is a paragraph";
        let after = "this is a great paragraph";
        assert_eq!(
            DiscourseDiff::new(before, after).side_by_side_markdown(),
            "<table class=\"markdown\"><tr><td class=\"--previous\">this is a paragraph</td><td class=\"--current\">this is a <ins>great </ins>paragraph</td></tr></table>"
        );
    }
}
