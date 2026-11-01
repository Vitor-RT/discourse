use crate::admin_reports::resolved_report::ResolvedReport;
use std::collections::{HashMap, HashSet};

pub trait SourceProvider: Send + Sync {
    fn source_name(&self) -> String {
        self.name()
    }
    fn name(&self) -> String {
        self.source_name()
    }
    fn label(&self) -> Option<String> {
        None
    }

    fn resolve_many(
        &self,
        _identifiers: &[String],
        _guardian: &dyn std::any::Any,
    ) -> HashMap<String, ResolvedReport> {
        HashMap::new()
    }

    fn fetch_many(
        &self,
        _identifiers: &[String],
        _guardian: &dyn std::any::Any,
        _filters: &HashMap<String, String>,
    ) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    fn prewarm(
        &self,
        _identifiers: &[String],
        _guardian: &dyn std::any::Any,
        _filters: &HashMap<String, String>,
    ) {
    }

    fn list_all(
        &self,
        _search: Option<&str>,
        _after: Option<&HashMap<String, String>>,
        _limit: Option<usize>,
    ) -> Vec<ResolvedReport> {
        Vec::new()
    }

    fn accessible_ids(
        &self,
        identifiers: &[String],
        guardian: &dyn std::any::Any,
    ) -> HashSet<String> {
        self.resolve_many(identifiers, guardian)
            .keys()
            .cloned()
            .collect()
    }

    fn accessible_report_identifiers(
        &self,
        identifiers: &[String],
        guardian: &dyn std::any::Any,
    ) -> HashSet<String> {
        self.accessible_ids(identifiers, guardian)
    }

    fn bulk_fetch(
        &self,
        identifiers: &[String],
        filters: &HashMap<String, String>,
        guardian: &dyn std::any::Any,
    ) -> HashMap<String, serde_json::Value> {
        self.fetch_many(identifiers, guardian, filters)
    }
}
