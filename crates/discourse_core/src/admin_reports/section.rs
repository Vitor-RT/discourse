use std::collections::HashMap;

use super::registry::Registry;
use super::resolved_report::ResolvedReport;

pub const DEFAULT_VISIBLE_CAP: usize = 20;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct SectionResult {
    pub items: Vec<SectionItem>,
}

#[derive(Debug, Clone)]
pub struct AdminDashboardReportModel {
    pub id: u64,
    pub source: String,
    pub identifier: String,
    pub position: usize,
    pub rows: usize,
    pub cols: usize,
}

<<<<<<< HEAD
pub trait AdminDashboardReportQueryStore {
    fn get_all_ordered_by_created_at_desc(&self) -> Vec<AdminDashboardReportModel>;
}

pub struct Section<'a, Q: AdminDashboardReportQueryStore> {
    pub registry: &'a Registry,
    pub store: &'a Q,
    pub visible_cap: usize,
    pub search_query: Option<String>,
}

impl<'a, Q: AdminDashboardReportQueryStore> Section<'a, Q> {
    pub fn new(
        visible_cap: usize,
        registry: &'a Registry,
        store: &'a Q,
        search_query: Option<String>,
    ) -> Self {
=======
impl<'a> Section<'a> {
    pub fn new(
        visible_cap: usize,
        registry: &'a Registry,
        store: &'a dyn AdminDashboardReportQueryStore,
        search: Option<String>,
    ) -> Self {
        let search = search.filter(|s| !s.trim().is_empty());
>>>>>>> origin/rust-migration-6812869939882800025
        Self {
            visible_cap,
            registry,
            store,
<<<<<<< HEAD
            search_query,
=======
            search,
>>>>>>> origin/rust-migration-6812869939882800025
        }
    }

    pub fn build<T: 'static>(&self, guardian: &T) -> SectionResult {
        let all_records = self.store.get_all_ordered_by_created_at_desc();
        let records = if all_records.len() > self.visible_cap {
            &all_records[..self.visible_cap]
        } else {
            &all_records[..]
        };

<<<<<<< HEAD
        let resolved_map = self.resolve_records(records, guardian);
=======
        let mut serialized: Vec<SectionItem> = items
            .into_iter()
            .map(|(record, resolved)| self.serialize(&resolved, &record))
            .collect();
>>>>>>> origin/rust-migration-6812869939882800025

        let mut survivors = Vec::new();
        for record in records {
            if let Some(resolved) = resolved_map.get(&record.id) {
                survivors.push((record, resolved));
            }
        }

<<<<<<< HEAD
=======
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

>>>>>>> origin/rust-migration-6812869939882800025
        survivors.sort_by_key(|(record, _)| record.position);

        let mut items = Vec::new();
        for (record, resolved) in survivors {
            items.push(self.serialize(resolved, record));
        }

        if let Some(ref query) = self.search_query {
            items = self.filter_by_search(items, query);
        }

        SectionResult { items }
    }

    fn resolve_records(
        &self,
        records: &[AdminDashboardReportModel],
        guardian: &dyn std::any::Any,
    ) -> HashMap<u64, ResolvedReport> {
<<<<<<< HEAD
=======
        // dispatch_per_source
>>>>>>> origin/rust-migration-6812869939882800025
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
    use serde_json::Value;

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
<<<<<<< HEAD
        ) -> HashMap<String, Value> {
=======
        ) -> HashMap<String, serde_json::Value> {
>>>>>>> origin/rust-migration-6812869939882800025
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
            self.records.clone()
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

<<<<<<< HEAD
=======
        // ordered by created_at DESC from store, but they should be sorted by position
>>>>>>> origin/rust-migration-6812869939882800025
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
