use crate::admin_reports::mounter::{AdminDashboardReportStore, MounterError};
use crate::admin_reports::registry::Registry;
use std::collections::HashSet;

pub struct LayoutItem {
    pub source: String,
    pub identifier: String,
    pub rows: Option<usize>,
    pub cols: Option<usize>,
}

pub struct LayoutUpdater<'a> {
    pub registry: &'a Registry,
    pub store: &'a mut dyn AdminDashboardReportStore,
}

impl<'a> LayoutUpdater<'a> {
    pub fn call(
        &mut self,
        items: &[LayoutItem],
        guardian: &dyn std::any::Any,
    ) -> Result<(), MounterError> {
        let mut grouped: std::collections::HashMap<&str, Vec<&LayoutItem>> =
            std::collections::HashMap::new();
        for item in items {
            grouped.entry(&item.source).or_default().push(item);
        }

        for (source, group) in grouped {
            let provider = self
                .registry
                .provider_for(source)
                .ok_or_else(|| MounterError::InvalidParameters("items".to_string()))?;

            let requested: HashSet<String> = group.iter().map(|i| i.identifier.clone()).collect();
            let requested_vec: Vec<String> = requested.iter().cloned().collect();
            let accessible = provider.accessible_ids(&requested_vec, guardian);

            if !requested.is_subset(&accessible) {
                return Err(MounterError::InvalidAccess);
            }
        }

        self.store.delete_all_records();

        for (index, item) in items.iter().enumerate() {
            self.store.create_layout_record(
                item.source.clone(),
                item.identifier.clone(),
                index,
                item.rows.unwrap_or(1),
                item.cols.unwrap_or(1),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_reports::mounter::AdminDashboardReportRecord;
    use crate::admin_reports::resolved_report::ResolvedReport;
    use crate::admin_reports::source_provider::SourceProvider;
    use std::collections::HashMap;

    struct DummyProvider {
        source: String,
        accessible: HashSet<String>,
    }

    impl SourceProvider for DummyProvider {
        fn source_name(&self) -> String {
            self.source.clone()
        }
        fn label(&self) -> Option<String> {
            None
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
            _s: Option<&str>,
            _a: Option<&HashMap<String, String>>,
            _l: Option<usize>,
        ) -> Vec<ResolvedReport> {
            vec![]
        }
        fn accessible_ids(
            &self,
            _identifiers: &[String],
            _guardian: &dyn std::any::Any,
        ) -> HashSet<String> {
            self.accessible.clone()
        }
    }

    struct MockStore {
        records: Vec<AdminDashboardReportRecord>,
    }

    impl AdminDashboardReportStore for MockStore {
        fn find_by(&self, source: &str, identifier: &str) -> Option<AdminDashboardReportRecord> {
            self.records
                .iter()
                .find(|r| r.source == source && r.identifier == identifier)
                .map(|r| AdminDashboardReportRecord {
                    source: r.source.clone(),
                    identifier: r.identifier.clone(),
                })
        }
        fn count(&self) -> usize {
            self.records.len()
        }
        fn create(
            &mut self,
            source: String,
            identifier: String,
        ) -> Result<AdminDashboardReportRecord, MounterError> {
            let record = AdminDashboardReportRecord {
                source: source.clone(),
                identifier: identifier.clone(),
            };
            self.records
                .push(AdminDashboardReportRecord { source, identifier });
            Ok(record)
        }
        fn delete_all(&mut self, source: &str, identifier: &str) {
            self.records
                .retain(|r| !(r.source == source && r.identifier == identifier));
        }
        fn delete_all_records(&mut self) {
            self.records.clear();
        }
        fn create_layout_record(
            &mut self,
            source: String,
            identifier: String,
            _position: usize,
            _rows: usize,
            _cols: usize,
        ) {
            let _ = self.create(source, identifier);
        }
    }

    #[test]
    fn test_layout_updater_success() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            source: "src1".to_string(),
            accessible: HashSet::from(["id1".to_string(), "id2".to_string()]),
        })]);

        let mut store = MockStore {
            records: vec![AdminDashboardReportRecord {
                source: "src1".to_string(),
                identifier: "id3".to_string(),
            }],
        };
        let mut updater = LayoutUpdater {
            registry: &registry,
            store: &mut store,
        };

        let items = vec![
            LayoutItem {
                source: "src1".to_string(),
                identifier: "id1".to_string(),
                rows: None,
                cols: None,
            },
            LayoutItem {
                source: "src1".to_string(),
                identifier: "id2".to_string(),
                rows: None,
                cols: None,
            },
        ];

        let result = updater.call(&items, &());
        assert!(result.is_ok());

        assert_eq!(updater.store.count(), 2);
        assert!(updater.store.find_by("src1", "id1").is_some());
        assert!(updater.store.find_by("src1", "id2").is_some());
        assert!(updater.store.find_by("src1", "id3").is_none());
    }

    #[test]
    fn test_layout_updater_invalid_access() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            source: "src1".to_string(),
            accessible: HashSet::from(["id1".to_string()]), // missing id2
        })]);

        let mut store = MockStore { records: vec![] };
        let mut updater = LayoutUpdater {
            registry: &registry,
            store: &mut store,
        };

        let items = vec![
            LayoutItem {
                source: "src1".to_string(),
                identifier: "id1".to_string(),
                rows: None,
                cols: None,
            },
            LayoutItem {
                source: "src1".to_string(),
                identifier: "id2".to_string(),
                rows: None,
                cols: None,
            },
        ];

        let result = updater.call(&items, &());
        assert_eq!(result.unwrap_err(), MounterError::InvalidAccess);
    }
}
