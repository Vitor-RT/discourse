use std::collections::{HashMap, HashSet};
use crate::admin_reports::resolved_report::ResolvedReport;

pub trait SourceProvider {
    fn source_name(&self) -> String;
    fn label(&self) -> Option<String>;

    // In Rust we'd usually use generic trait or trait objects for Guardian, but for now we can mock it as an opaque type or just generic parameters, or perhaps we just need the trait definition.
    // For now we'll pass a dummy `&Guardian` but we don't have Guardian in Rust yet.
    // Wait, let me just check how they are used.

    fn resolve_many(&self, identifiers: &[String], guardian: &dyn std::any::Any) -> HashMap<String, ResolvedReport>;

    fn fetch_many(&self, identifiers: &[String], guardian: &dyn std::any::Any, filters: &HashMap<String, String>) -> HashMap<String, serde_json::Value>;

    fn prewarm(&self, _identifiers: &[String], _guardian: &dyn std::any::Any, _filters: &HashMap<String, String>) {}

    fn list_all(
        &self,
        search: Option<&str>,
        after: Option<&HashMap<String, String>>,
        limit: Option<usize>
    ) -> Vec<ResolvedReport>;

    fn accessible_ids(&self, identifiers: &[String], guardian: &dyn std::any::Any) -> HashSet<String> {
        self.resolve_many(identifiers, guardian).keys().cloned().collect()
    }
}
