use std::collections::{HashMap, HashSet};

use super::layout_updater::{AdminDashboardReportRecord, AdminDashboardReportStore};
use super::registry::Registry;

pub const DEFAULT_VISIBLE_CAP: usize = 20;

#[derive(Debug, PartialEq, Eq)]
pub enum MounterError {
    CapReached,
    InvalidAccess,
}

impl std::fmt::Display for MounterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CapReached => write!(f, "Cap reached"),
            Self::InvalidAccess => write!(f, "Invalid access"),
        }
    }
}

impl std::error::Error for MounterError {}

pub struct Mounter<'a, S: AdminDashboardReportStore> {
    pub registry: &'a Registry,
    pub store: &'a mut S,
    pub visible_cap: usize,
}

impl<'a, S: AdminDashboardReportStore> Mounter<'a, S> {
    pub fn new(registry: &'a Registry, store: &'a mut S) -> Self {
        Self {
            registry,
            store,
            visible_cap: DEFAULT_VISIBLE_CAP,
        }
    }

    pub fn mount<T: 'static>(
        &mut self,
        source_name: &str,
        identifier: &str,
        user: &T,
    ) -> Result<(), MounterError> {
        let provider = self
            .registry
            .provider_for(source_name)
            .ok_or(MounterError::InvalidAccess)?;

        let accessible = provider.accessible_report_identifiers(&[identifier.to_string()], user);
        if !accessible.contains(identifier) {
            return Err(MounterError::InvalidAccess);
        }

        if self.store.count() >= self.visible_cap {
            return Err(MounterError::CapReached);
        }

        if self.store.find_by(source_name, identifier).is_none() {
            let _ = self
                .store
                .create(source_name.to_string(), identifier.to_string());
        }

        Ok(())
    }

    pub fn unmount(&mut self, source_name: &str, identifier: &str) {
        self.store
            .delete_all(source_name, &[identifier.to_string()]);
    }
}

#[cfg(test)]
mod tests {
    use super::super::source_provider::SourceProvider;
    use super::*;
    use serde_json::Value;

    struct DummyProvider {
        source: String,
        accessible: HashSet<String>,
    }

    impl SourceProvider for DummyProvider {
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
        }

        fn bulk_fetch(
            &self,
            _identifiers: &[String],
            _filters: &HashMap<String, String>,
            _user: &dyn std::any::Any,
        ) -> HashMap<String, Value> {
            HashMap::new()
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

        fn create(
            &mut self,
            source: String,
            identifier: String,
        ) -> Result<AdminDashboardReportRecord, MounterError> {
            let rec = AdminDashboardReportRecord {
                source: source.clone(),
                identifier: identifier.clone(),
            };
            self.records
                .push(AdminDashboardReportRecord { source, identifier });
            Ok(rec)
        }

        fn count(&self) -> usize {
            self.records.len()
        }

        fn delete_all(&mut self, source: &str, identifiers: &[String]) {
            self.records
                .retain(|r| !(r.source == source && identifiers.contains(&r.identifier)));
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
            visible_cap: 1,
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
