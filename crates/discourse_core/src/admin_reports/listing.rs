use crate::admin_reports::registry::Registry;
use crate::admin_reports::resolved_report::ResolvedReport;
use serde::Serialize;
use std::collections::HashMap;

const PAGE_SIZE: usize = 30;

#[derive(Serialize)]
pub struct ProviderSummary {
    pub source: String,
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct ListingResult {
    pub providers: Vec<ProviderSummary>,
    pub items: Vec<ResolvedReport>,
    pub has_more: bool,
    pub cursor: Option<HashMap<String, String>>,
}

pub struct Listing<'a> {
    pub registry: &'a Registry,
    pub cursor: Option<HashMap<String, String>>,
    pub search: Option<String>,
}

impl<'a> Listing<'a> {
    pub fn new(
        registry: &'a Registry,
        cursor_raw: Option<HashMap<String, String>>,
        search_raw: Option<String>,
    ) -> Self {
        let cursor = Listing::normalize_cursor(cursor_raw);
        let search = search_raw.filter(|s| !s.trim().is_empty());
        Self {
            registry,
            cursor,
            search,
        }
    }

    pub fn call(&self) -> ListingResult {
        let search_ref = self.search.as_deref();

        let mut merged: Vec<ResolvedReport> = self
            .registry
            .providers()
            .iter()
            .flat_map(|provider| {
                provider.list_all(search_ref, self.cursor.as_ref(), Some(PAGE_SIZE + 1))
            })
            .collect();

        merged.sort_by(|a, b| {
            a.title
                .to_lowercase()
                .cmp(&b.title.to_lowercase())
                .then(a.key().cmp(&b.key()))
        });

        let has_more = merged.len() > PAGE_SIZE;
        let page = if has_more {
            &merged[0..PAGE_SIZE]
        } else {
            &merged[..]
        };

        let cursor_out = if has_more {
            page.last().map(Listing::cursor_for)
        } else {
            None
        };

        ListingResult {
            providers: self.provider_summaries(),
            items: page.to_vec(),
            has_more,
            cursor: cursor_out,
        }
    }

    fn provider_summaries(&self) -> Vec<ProviderSummary> {
        self.registry
            .providers()
            .iter()
            .map(|provider| ProviderSummary {
                source: provider.source_name(),
                label: provider.label(),
            })
            .collect()
    }

    fn normalize_cursor(raw: Option<HashMap<String, String>>) -> Option<HashMap<String, String>> {
        let map = raw?;
        let title = map.get("title")?;
        let key = map.get("key")?;
        if title.trim().is_empty() || key.trim().is_empty() {
            return None;
        }

        let mut out = HashMap::new();
        out.insert("title".to_string(), title.to_string());
        out.insert("key".to_string(), key.to_string());
        Some(out)
    }

    fn cursor_for(report: &ResolvedReport) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("title".to_string(), report.title.clone());
        map.insert("key".to_string(), report.key());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_reports::source_provider::SourceProvider;

    struct DummyProvider {
        name: String,
        reports: Vec<ResolvedReport>,
    }

    impl SourceProvider for DummyProvider {
        fn source_name(&self) -> String {
            self.name.clone()
        }
        fn label(&self) -> Option<String> {
            Some(format!("{} label", self.name))
        }
        fn resolve_many(
            &self,
            _ids: &[String],
            _g: &dyn std::any::Any,
        ) -> HashMap<String, ResolvedReport> {
            HashMap::new()
        }
        fn fetch_many(
            &self,
            _ids: &[String],
            _g: &dyn std::any::Any,
            _f: &HashMap<String, String>,
        ) -> HashMap<String, serde_json::Value> {
            HashMap::new()
        }

        fn list_all(
            &self,
            search: Option<&str>,
            _after: Option<&HashMap<String, String>>,
            limit: Option<usize>,
        ) -> Vec<ResolvedReport> {
            let mut out = self.reports.clone();
            if let Some(s) = search {
                out.retain(|r| {
                    r.title.contains(s) || r.description.as_deref().unwrap_or("").contains(s)
                });
            }
            if let Some(l) = limit {
                out.truncate(l);
            }
            out
        }
    }

    #[test]
    fn test_listing_pagination() {
        let mut reports = Vec::new();
        for i in 0..35 {
            reports.push(ResolvedReport {
                source: "src1".to_string(),
                identifier: format!("id{}", i),
                title: format!("Title {:02}", i),
                description: None,
                label: None,
                url: None,
            });
        }

        let provider = Box::new(DummyProvider {
            name: "src1".to_string(),
            reports,
        });

        let registry = Registry::new(vec![provider]);
        let listing = Listing::new(&registry, None, None);

        let result = listing.call();
        assert_eq!(result.items.len(), 30);
        assert_eq!(result.has_more, true);
        assert!(result.cursor.is_some());

        assert_eq!(result.providers.len(), 1);
        assert_eq!(result.providers[0].source, "src1");
        assert_eq!(result.providers[0].label.as_deref(), Some("src1 label"));
    }
}
