use crate::admin_reports::source_provider::SourceProvider;
use std::collections::HashMap;

pub struct Registry {
    providers: Vec<Box<dyn SourceProvider>>,
}

impl Registry {
    pub fn new(providers: Vec<Box<dyn SourceProvider>>) -> Self {
        Self { providers }
    }

    pub fn providers(&self) -> &Vec<Box<dyn SourceProvider>> {
        &self.providers
    }

    pub fn provider_for(&self, source_name: &str) -> Option<&Box<dyn SourceProvider>> {
<<<<<<< HEAD
        self.providers
            .iter()
            .find(|p| p.source_name() == source_name)
    }

    pub fn dispatch_per_source<T, F, R>(
        &self,
        items: &[T],
        source_fn: impl Fn(&T) -> String,
        mut handler: F,
    ) -> HashMap<String, R>
=======
        self.providers.iter().find(|p| p.source_name() == source_name)
    }

    pub fn dispatch_per_source<T, F, R>(&self, items: &[T], source_fn: impl Fn(&T) -> String, mut handler: F) -> HashMap<String, R>
>>>>>>> origin/rust-migration-7847495834899956704
    where
        F: FnMut(&Box<dyn SourceProvider>, &[&T]) -> R,
    {
        let mut grouped: HashMap<String, Vec<&T>> = HashMap::new();
        for item in items {
            grouped.entry(source_fn(item)).or_default().push(item);
        }

        let mut result = HashMap::new();
        for (source, group) in grouped {
            if let Some(provider) = self.provider_for(&source) {
                result.insert(source, handler(provider, &group));
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_reports::resolved_report::ResolvedReport;

    struct DummyProvider {
        name: String,
    }

    impl SourceProvider for DummyProvider {
        fn source_name(&self) -> String {
            self.name.clone()
        }
<<<<<<< HEAD
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
=======
        fn label(&self) -> Option<String> { None }
        fn resolve_many(&self, _ids: &[String], _guardian: &dyn std::any::Any) -> HashMap<String, ResolvedReport> { HashMap::new() }
        fn fetch_many(&self, _ids: &[String], _guardian: &dyn std::any::Any, _filters: &HashMap<String, String>) -> HashMap<String, serde_json::Value> { HashMap::new() }
        fn list_all(&self, _search: Option<&str>, _after: Option<&HashMap<String, String>>, _limit: Option<usize>) -> Vec<ResolvedReport> { vec![] }
>>>>>>> origin/rust-migration-7847495834899956704
    }

    #[test]
    fn test_provider_for() {
<<<<<<< HEAD
        let registry = Registry::new(vec![Box::new(DummyProvider {
            name: "test_source".to_string(),
        })]);
=======
        let registry = Registry::new(vec![Box::new(DummyProvider { name: "test_source".to_string() })]);
>>>>>>> origin/rust-migration-7847495834899956704
        assert!(registry.provider_for("test_source").is_some());
        assert!(registry.provider_for("unknown").is_none());
    }

    #[test]
    fn test_dispatch_per_source() {
        let registry = Registry::new(vec![
<<<<<<< HEAD
            Box::new(DummyProvider {
                name: "source1".to_string(),
            }),
            Box::new(DummyProvider {
                name: "source2".to_string(),
            }),
=======
            Box::new(DummyProvider { name: "source1".to_string() }),
            Box::new(DummyProvider { name: "source2".to_string() }),
>>>>>>> origin/rust-migration-7847495834899956704
        ]);

        let items = vec![
            ("source1".to_string(), "id1"),
            ("source1".to_string(), "id2"),
            ("source2".to_string(), "id3"),
            ("unknown".to_string(), "id4"),
        ];

        let result = registry.dispatch_per_source(
            &items,
            |item| item.0.clone(),
<<<<<<< HEAD
            |provider, group| format!("{}:{}", provider.source_name(), group.len()),
=======
            |provider, group| {
                format!("{}:{}", provider.source_name(), group.len())
            },
>>>>>>> origin/rust-migration-7847495834899956704
        );

        assert_eq!(result.get("source1").unwrap(), "source1:2");
        assert_eq!(result.get("source2").unwrap(), "source2:1");
        assert!(result.get("unknown").is_none());
    }
}
