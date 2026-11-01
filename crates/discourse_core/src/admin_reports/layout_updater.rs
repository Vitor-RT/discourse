<<<<<<< HEAD
use std::collections::HashMap;
=======
use crate::admin_reports::mounter::{AdminDashboardReportStore, MounterError};
use crate::admin_reports::registry::Registry;
use std::collections::HashSet;
>>>>>>> origin/rust-migration-6812869939882800025

use super::mounter::MounterError;
use super::registry::Registry;

#[derive(Debug, Clone)]
pub struct LayoutItem {
    pub source: String,
    pub identifier: String,
    pub rows: Option<usize>,
    pub cols: Option<usize>,
}

pub struct AdminDashboardReportRecord {
    pub source: String,
    pub identifier: String,
}

<<<<<<< HEAD
pub trait AdminDashboardReportStore {
    fn find_by(&self, source: &str, identifier: &str) -> Option<&AdminDashboardReportRecord>;
    fn create(&mut self, source: String, identifier: String) -> AdminDashboardReportRecord;
    fn count(&self) -> usize;
    fn delete_all(&mut self, source: &str, identifiers: &[String]);
    fn create_layout_record(
        &mut self,
        source: String,
        identifier: String,
        position: usize,
        rows: usize,
        cols: usize,
    );
}

pub struct LayoutUpdater<'a, S: AdminDashboardReportStore> {
    pub registry: &'a Registry,
    pub store: &'a mut S,
}

impl<'a, S: AdminDashboardReportStore> LayoutUpdater<'a, S> {
    pub fn new(registry: &'a Registry, store: &'a mut S) -> Self {
        Self { registry, store }
    }

    pub fn call<T: 'static>(&mut self, items: &[LayoutItem], user: &T) -> Result<(), MounterError> {
        let mut grouped: HashMap<String, Vec<&LayoutItem>> = HashMap::new();
=======
impl<'a> LayoutUpdater<'a> {
    pub fn call(
        &mut self,
        items: &[LayoutItem],
        guardian: &dyn std::any::Any,
    ) -> Result<(), MounterError> {
        let mut grouped: std::collections::HashMap<&str, Vec<&LayoutItem>> =
            std::collections::HashMap::new();
>>>>>>> origin/rust-migration-6812869939882800025
        for item in items {
            grouped.entry(item.source.clone()).or_default().push(item);
        }

<<<<<<< HEAD
        for (source_name, source_items) in &grouped {
            let provider = self
                .registry
                .provider_for(source_name)
                .ok_or(MounterError::InvalidAccess)?;
=======
        for (source, group) in grouped {
            let provider = self
                .registry
                .provider_for(source)
                .ok_or_else(|| MounterError::InvalidParameters("items".to_string()))?;
>>>>>>> origin/rust-migration-6812869939882800025

            let identifiers: Vec<String> = source_items.iter().map(|item| item.identifier.clone()).collect();
            let accessible = provider.accessible_report_identifiers(&identifiers, user);

            for item in source_items {
                if !accessible.contains(&item.identifier) {
                    return Err(MounterError::InvalidAccess);
                }
            }
        }

        for (source_name, source_items) in grouped {
            let identifiers: Vec<String> = source_items.iter().map(|item| item.identifier.clone()).collect();
            self.store.delete_all(&source_name, &identifiers);

            for (index, item) in source_items.into_iter().enumerate() {
                self.store.create_layout_record(
                    item.source.clone(),
                    item.identifier.clone(),
                    index + 1,
                    item.rows.unwrap_or(1),
                    item.cols.unwrap_or(1),
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< HEAD
    use super::super::source_provider::SourceProvider;
    use std::collections::HashSet;
=======
    use crate::admin_reports::mounter::AdminDashboardReportRecord;
    use crate::admin_reports::resolved_report::ResolvedReport;
    use crate::admin_reports::source_provider::SourceProvider;
    use std::collections::HashMap;
>>>>>>> origin/rust-migration-6812869939882800025

    struct DummyProvider {
        source: String,
        accessible: HashSet<String>,
    }

    impl SourceProvider for DummyProvider {
<<<<<<< HEAD
        fn name(&self) -> String {
            self.source.clone()
        }

        fn accessible_report_identifiers(
            &self,
            identifiers: &[String],
            _user: &dyn std::any::Any,
        ) -> HashSet<String> {
            identifiers
                .iter()
                .filter(|id| self.accessible.contains(*id))
                .cloned()
                .collect()
=======
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
>>>>>>> origin/rust-migration-6812869939882800025
        }
    }

    struct MockStore {
        records: Vec<AdminDashboardReportRecord>,
    }

    impl AdminDashboardReportStore for MockStore {
<<<<<<< HEAD
        fn find_by(&self, source: &str, identifier: &str) -> Option<&AdminDashboardReportRecord> {
            self.records
                .iter()
                .find(|r| r.source == source && r.identifier == identifier)
        }

        fn create(&mut self, source: String, identifier: String) -> AdminDashboardReportRecord {
            let rec = AdminDashboardReportRecord { source, identifier };
            self.records.push(AdminDashboardReportRecord {
                source: rec.source.clone(),
                identifier: rec.identifier.clone(),
            });
            rec
        }

        fn count(&self) -> usize {
            self.records.len()
=======
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
>>>>>>> origin/rust-migration-6812869939882800025
        }

        fn delete_all(&mut self, source: &str, identifiers: &[String]) {
            self.records.retain(|r| !(r.source == source && identifiers.contains(&r.identifier)));
        }
<<<<<<< HEAD

=======
>>>>>>> origin/rust-migration-6812869939882800025
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

        assert_eq!(updater.store.count(), 3);
        assert!(updater.store.find_by("src1", "id1").is_some());
        assert!(updater.store.find_by("src1", "id2").is_some());
        assert!(updater.store.find_by("src1", "id3").is_some());
    }

    #[test]
    fn test_layout_updater_invalid_access() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            source: "src1".to_string(),
            accessible: HashSet::from(["id1".to_string()]),
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
