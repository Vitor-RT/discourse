use crate::admin_reports::registry::Registry;

#[derive(Debug, PartialEq, Eq)]
pub enum MounterError {
    InvalidParameters(String),
    InvalidAccess,
    CapReached,
}

#[derive(Debug)]
pub struct AdminDashboardReportRecord {
    pub source: String,
    pub identifier: String,
}

pub trait AdminDashboardReportStore {
    fn find_by(&self, source: &str, identifier: &str) -> Option<AdminDashboardReportRecord>;
    fn count(&self) -> usize;
    fn create(
        &mut self,
        source: String,
        identifier: String,
    ) -> Result<AdminDashboardReportRecord, MounterError>;
    fn delete_all(&mut self, source: &str, identifier: &str);
    fn delete_all_records(&mut self);
    fn create_layout_record(
        &mut self,
        source: String,
        identifier: String,
        position: usize,
        rows: usize,
        cols: usize,
    );
}

pub struct Mounter<'a> {
    pub visible_cap: usize,
    pub registry: &'a Registry,
    pub store: &'a mut dyn AdminDashboardReportStore,
}

impl<'a> Mounter<'a> {
    pub fn mount(
        &mut self,
        source: &str,
        identifier: &str,
        guardian: &dyn std::any::Any,
    ) -> Result<AdminDashboardReportRecord, MounterError> {
        let provider = self
            .registry
            .provider_for(source)
            .ok_or_else(|| MounterError::InvalidParameters("source".to_string()))?;

        let accessible = provider.accessible_ids(&[identifier.to_string()], guardian);
        if !accessible.contains(identifier) {
            return Err(MounterError::InvalidAccess);
        }

        if let Some(existing) = self.store.find_by(source, identifier) {
            return Ok(existing);
        }

        if self.store.count() >= self.visible_cap {
            return Err(MounterError::CapReached);
        }

        self.store
            .create(source.to_string(), identifier.to_string())
    }

    pub fn unmount(&mut self, source: &str, identifier: &str) {
        self.store.delete_all(source, identifier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_reports::resolved_report::ResolvedReport;
    use crate::admin_reports::source_provider::SourceProvider;
    use std::collections::HashMap;
    use std::collections::HashSet;

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
            _guardian: &dyn std::any::Any,
        ) -> HashMap<String, ResolvedReport> {
            HashMap::new()
        }
        fn fetch_many(
            &self,
            _ids: &[String],
            _guardian: &dyn std::any::Any,
            _filters: &HashMap<String, String>,
        ) -> HashMap<String, serde_json::Value> {
            HashMap::new()
        }
        fn list_all(
            &self,
            _search: Option<&str>,
            _after: Option<&HashMap<String, String>>,
            _limit: Option<usize>,
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
    fn test_mount_success() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            source: "src1".to_string(),
            accessible: HashSet::from(["id1".to_string()]),
        })]);

        let mut store = MockStore { records: vec![] };
        let mut mounter = Mounter {
            visible_cap: 3,
            registry: &registry,
            store: &mut store,
        };

        let result = mounter.mount("src1", "id1", &());
        assert!(result.is_ok());
        assert_eq!(mounter.store.count(), 1);
    }

    #[test]
    fn test_mount_cap_reached() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            source: "src1".to_string(),
            accessible: HashSet::from(["id1".to_string()]),
        })]);

        let mut store = MockStore {
            records: vec![AdminDashboardReportRecord {
                source: "src2".to_string(),
                identifier: "id2".to_string(),
            }],
        };

        let mut mounter = Mounter {
            visible_cap: 1, // Cap is 1, and we already have 1
            registry: &registry,
            store: &mut store,
        };

        let result = mounter.mount("src1", "id1", &());
        assert_eq!(result.unwrap_err(), MounterError::CapReached);
    }

    #[test]
    fn test_mount_invalid_access() {
        let registry = Registry::new(vec![Box::new(DummyProvider {
            source: "src1".to_string(),
            accessible: HashSet::new(),
        })]);

        let mut store = MockStore { records: vec![] };
        let mut mounter = Mounter {
            visible_cap: 3,
            registry: &registry,
            store: &mut store,
        };

        let result = mounter.mount("src1", "id1", &());
        assert_eq!(result.unwrap_err(), MounterError::InvalidAccess);
    }

    #[test]
    fn test_unmount() {
        let registry = Registry::new(vec![]);
        let mut store = MockStore {
            records: vec![AdminDashboardReportRecord {
                source: "src1".to_string(),
                identifier: "id1".to_string(),
            }],
        };

        let mut mounter = Mounter {
            visible_cap: 3,
            registry: &registry,
            store: &mut store,
        };
        mounter.unmount("src1", "id1");
        assert_eq!(mounter.store.count(), 0);
    }
}
