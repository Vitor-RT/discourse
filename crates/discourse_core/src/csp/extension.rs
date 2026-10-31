use crate::csp::Directives;
use url::Url;

pub struct Extension;

impl Extension {
    pub fn build_theme_extension(entries: Vec<String>) -> Directives {
        let mut extension = Directives::new();
        for entry in entries {
            if let Some((directive, source)) = entry.split_once(':') {
                let directive = directive.trim().to_string();
                let source = source.trim().to_string();
                extension.entry(directive).or_insert_with(Vec::new).push(source);
            }
        }
        extension
    }

    pub fn auto_script_src_extension(html_docs: &str, cdn_url: Option<&str>) -> Directives {
        let mut extension = Directives::new();
        let script_src = extension.entry("script_src".to_string()).or_insert_with(Vec::new);

        let mut start = 0;
        while let Some(script_idx) = html_docs[start..].find("<script") {
            let tag_start = start + script_idx;
            if let Some(tag_end) = html_docs[tag_start..].find('>') {
                let tag_content = &html_docs[tag_start..tag_start + tag_end];
                if let Some(src_idx) = tag_content.find("src=") {
                    let mut url_start = src_idx + 4;
                    let quote_char = tag_content[url_start..].chars().next();
                    if let Some(q) = quote_char {
                        if q == '"' || q == '\'' {
                            url_start += 1;
                            if let Some(url_end) = tag_content[url_start..].find(q) {
                                let src = &tag_content[url_start..url_start + url_end];

                                let ignore_cdn = if let Some(cdn) = cdn_url {
                                    src.starts_with(cdn)
                                } else {
                                    false
                                };

                                if !ignore_cdn {
                                    let parse_src = if src.starts_with("//") {
                                        format!("http:{}", src)
                                    } else {
                                        src.to_string()
                                    };

                                    if let Ok(mut parsed_url) = Url::parse(&parse_src) {
                                        if parsed_url.host_str().is_some() && !parsed_url.path().is_empty() {
                                            parsed_url.set_query(None);
                                            let mut uri_string = parsed_url.to_string();

                                            if src.starts_with("//") {
                                                uri_string = uri_string.replace("http://", "");
                                            }

                                            script_src.push(uri_string);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                start = tag_start + tag_end;
            } else {
                break;
            }
        }

        extension
    }
}
