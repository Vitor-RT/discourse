use crate::admin_reports::registry::Registry;
use crate::admin_reports::resolved_report::ResolvedReport;
use std::collections::HashMap;

// Using a placeholder struct to represent AdminDashboardReport that we fetch from DB
#[derive(Debug, Clone)]
pub struct AdminDashboardReportModel {
    pub id: u64,
    pub source: String,
    pub identifier: String,
    pub position: usize,
    pub rows: usize,
    pub cols: usize,
}

pub trait AdminDashboardReportQueryStore {
    // Should return records ordered by created_at DESC
    fn get_all_ordered_by_created_at_desc(&self) -> Vec<AdminDashboardReportModel>;
}

// Result struct representing a serialized resolved item with rows and cols
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SectionItem {
    pub source: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub label: Option<String>,
    pub url: Option<String>,
    pub key: String,
    pub rows: usize,
    pub cols: usize,
}

pub struct SectionResult {
    pub items: Vec<SectionItem>,
}

pub struct Section<'a> {
    pub visible_cap: usize,
    pub registry: &'a Registry,
    pub store: &'a dyn AdminDashboardReportQueryStore,
    pub search: Option<String>,
}

impl<'a> Section<'a> {
    pub fn new(
        visible_cap: usize,
        registry: &'a Registry,
        store: &'a dyn AdminDashboardReportQueryStore,
        search: Option<String>,
    ) -> Self {
        let search = search.filter(|s| !s.trim().is_empty());
        Self {
            visible_cap,
            registry,
            store,
            search,
        }
    }

    pub fn build(&self, guardian: &dyn std::any::Any) -> SectionResult {
        let items = self.visible_items(guardian);

        let mut serialized: Vec<SectionItem> = items
            .into_iter()
            .map(|(record, resolved)| self.serialize(&resolved, &record))
            .collect();

        if let Some(ref search_query) = self.search {
            serialized = self.filter_by_search(serialized, search_query);
        }

        SectionResult { items: serialized }
    }

    fn visible_items(
        &self,
        guardian: &dyn std::any::Any,
    ) -> Vec<(AdminDashboardReportModel, ResolvedReport)> {
        let records = self.store.get_all_ordered_by_created_at_desc();

        let resolved_by_record_id = self.resolve_records(&records, guardian);

        // Filter those that were resolved, take up to VISIBLE_CAP, then sort by position
        let mut survivors: Vec<(AdminDashboardReportModel, ResolvedReport)> = records
            .into_iter()
            .filter_map(|record| {
                resolved_by_record_id
                    .get(&record.id)
                    .map(|obj| (record, obj.clone()))
            })
            .take(self.visible_cap)
            .collect();

        survivors.sort_by_key(|(record, _)| record.position);
        survivors
    }

    fn resolve_records(
        &self,
        records: &[AdminDashboardReportModel],
        guardian: &dyn std::any::Any,
    ) -> HashMap<u64, ResolvedReport> {
        // dispatch_per_source
        let per_source = self.registry.dispatch_per_source(
            records,
            |record| record.source.clone(),
            |provider, group| {
                let identifiers: Vec<String> = group.iter().map(|r| r.identifier.clone()).collect();
                provider.resolve_many(&identifiers, guardian)
            },
        );

        let mut resolved = HashMap::new();
        for record in records {
            if let Some(source_map) = per_source.get(&record.source) {
                if let Some(report) = source_map.get(&record.identifier) {
                    resolved.insert(record.id, report.clone());
                }
            }
        }

        resolved
    }

    fn serialize(
        &self,
        resolved: &ResolvedReport,
        record: &AdminDashboardReportModel,
    ) -> SectionItem {
        SectionItem {
            source: resolved.source.clone(),
            identifier: resolved.identifier.clone(),
            title: resolved.title.clone(),
            description: resolved.description.clone(),
            label: resolved.label.clone(),
            url: resolved.url.clone(),
            key: resolved.key(),
            rows: record.rows,
            cols: record.cols,
        }
    }

    fn filter_by_search(&self, items: Vec<SectionItem>, query: &str) -> Vec<SectionItem> {
        let query_lower = query.to_lowercase();
        items
            .into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower)
                    || item
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query_lower)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_reports::source_provider::SourceProvider;

    struct DummyProvider {
        name: String,
        label_val: Option<String>,
    }

    impl SourceProvider for DummyProvider {
        fn source_name(&self) -> String {
            self.name.clone()
        }
        fn label(&self) -> Option<String> {
            self.label_val.clone()
        }
        fn resolve_many(
            &self,
            ids: &[String],
            _g: &dyn std::any::Any,
        ) -> HashMap<String, ResolvedReport> {
            let mut out = HashMap::new();
            for id in ids {
                if !id.starts_with("missing_") {
                    out.insert(
                        id.clone(),
                        ResolvedReport {
                            source: self.name.clone(),
                            identifier: id.clone(),
                            title: format!("Title for {}", id),
                            description: Some(format!("Desc for {}", id)),
                            label: self.label_val.clone(),
                            url: Some(format!("/{}/{}", self.name, id)),
                        },
                    );
                }
            }
            out
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
            _s: Option<&str>,
            _a: Option<&HashMap<String, String>>,
            _l: Option<usize>,
        ) -> Vec<ResolvedReport> {
            vec![]
        }
    }

    struct MockQueryStore {
        records: Vec<AdminDashboardReportModel>,
    }

    impl AdminDashboardReportQueryStore for MockQueryStore {
        fn get_all_ordered_by_created_at_desc(&self) -> Vec<AdminDashboardReportModel> {
            self.records.clone() // Assume already ordered
        }
    }

    #[test]
    fn test_section_build_empty() {
        let registry = Registry::new(vec![]);
        let store = MockQueryStore { records: vec![] };
        let section = Section::new(10, &registry, &store, None);

        let result = section.build(&());
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn test_section_build_ordering() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            name: "fake".to_string(),
            label_val: Some("Fake".to_string()),
        })]);

        // ordered by created_at DESC from store, but they should be sorted by position
        let store = MockQueryStore {
            records: vec![
                AdminDashboardReportModel {
                    id: 1,
                    source: "fake".to_string(),
                    identifier: "a".to_string(),
                    position: 1,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 2,
                    source: "fake".to_string(),
                    identifier: "b".to_string(),
                    position: 0,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 3,
                    source: "fake".to_string(),
                    identifier: "c".to_string(),
                    position: 2,
                    rows: 1,
                    cols: 1,
                },
            ],
        };

        let section = Section::new(10, &registry, &store, None);
        let result = section.build(&());

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].identifier, "b");
        assert_eq!(result.items[1].identifier, "a");
        assert_eq!(result.items[2].identifier, "c");
    }

    #[test]
    fn test_section_build_drops_missing() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            name: "fake".to_string(),
            label_val: Some("Fake".to_string()),
        })]);

        let store = MockQueryStore {
            records: vec![
                AdminDashboardReportModel {
                    id: 1,
                    source: "fake".to_string(),
                    identifier: "good".to_string(),
                    position: 0,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 2,
                    source: "unregistered".to_string(),
                    identifier: "orphan".to_string(),
                    position: 1,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 3,
                    source: "fake".to_string(),
                    identifier: "missing_one".to_string(),
                    position: 2,
                    rows: 1,
                    cols: 1,
                },
            ],
        };

        let section = Section::new(10, &registry, &store, None);
        let result = section.build(&());

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].identifier, "good");
    }

    #[test]
    fn test_section_build_caps() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            name: "fake".to_string(),
            label_val: Some("Fake".to_string()),
        })]);

        let store = MockQueryStore {
            records: vec![
                AdminDashboardReportModel {
                    id: 1,
                    source: "fake".to_string(),
                    identifier: "r_0".to_string(),
                    position: 0,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 2,
                    source: "fake".to_string(),
                    identifier: "r_1".to_string(),
                    position: 1,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 3,
                    source: "fake".to_string(),
                    identifier: "r_2".to_string(),
                    position: 2,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 4,
                    source: "fake".to_string(),
                    identifier: "r_3".to_string(),
                    position: 3,
                    rows: 1,
                    cols: 1,
                },
                AdminDashboardReportModel {
                    id: 5,
                    source: "fake".to_string(),
                    identifier: "r_4".to_string(),
                    position: 4,
                    rows: 1,
                    cols: 1,
                },
            ],
        };

        let section = Section::new(3, &registry, &store, None);
        let result = section.build(&());

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].identifier, "r_0");
        assert_eq!(result.items[1].identifier, "r_1");
        assert_eq!(result.items[2].identifier, "r_2");
    }
}
