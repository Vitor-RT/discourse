use serde_json::Value;
use std::collections::HashMap;

use super::registry::Registry;

#[derive(Debug, Clone)]
pub struct BulkFetchItem {
    pub source: String,
    pub identifier: String,
}

#[derive(Debug, Clone)]
pub struct BulkFetchResultItem {
    pub source: String,
    pub identifier: String,
    pub error: bool,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct BulkFetchResult {
    pub items: Vec<BulkFetchResultItem>,
}

pub struct BulkFetch<'a> {
    pub registry: &'a Registry,
}

impl<'a> BulkFetch<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self { registry }
    }

    pub fn call<T: 'static>(
        &self,
        items: &[BulkFetchItem],
        filters: &HashMap<String, String>,
        user: &T,
    ) -> BulkFetchResult {
        let mut grouped: HashMap<String, Vec<&BulkFetchItem>> = HashMap::new();
        for item in items {
            grouped.entry(item.source.clone()).or_default().push(item);
        }

        let mut results = Vec::new();

        for (source_name, source_items) in grouped {
            if let Some(provider) = self.registry.provider_for(&source_name) {
                let identifiers: Vec<String> = source_items
                    .iter()
                    .map(|item| item.identifier.clone())
                    .collect();

                let fetched_reports =
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        provider.bulk_fetch(&identifiers, filters, user)
                    })) {
                        Ok(reports) => reports,
                        Err(_) => HashMap::new(),
                    };

                for item in source_items {
                    if let Some(data) = fetched_reports.get(&item.identifier) {
                        let is_error = data.get("error").is_some();
                        results.push(BulkFetchResultItem {
                            source: item.source.clone(),
                            identifier: item.identifier.clone(),
                            error: is_error,
                            data: Some(data.clone()),
                        });
                    } else {
                        results.push(BulkFetchResultItem {
                            source: item.source.clone(),
                            identifier: item.identifier.clone(),
                            error: false,
                            data: None,
                        });
                    }
                }
            } else {
                for item in source_items {
                    results.push(BulkFetchResultItem {
                        source: item.source.clone(),
                        identifier: item.identifier.clone(),
                        error: false,
                        data: None,
                    });
                }
            }
        }

        BulkFetchResult { items: results }
    }
}

#[cfg(test)]
mod tests {
    use super::super::source_provider::SourceProvider;
    use super::*;
    use serde_json::json;

    struct DummyProvider {
        name: String,
        payloads: HashMap<String, Value>,
    }

    impl SourceProvider for DummyProvider {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn bulk_fetch(
            &self,
            identifiers: &[String],
            _filters: &HashMap<String, String>,
            _user: &dyn std::any::Any,
        ) -> HashMap<String, Value> {
            let mut result = HashMap::new();
            for id in identifiers {
                if let Some(val) = self.payloads.get(id) {
                    result.insert(id.clone(), val.clone());
                }
            }
            result
        }
    }

    #[test]
    fn test_bulk_fetch_call() {
        let mut payloads = HashMap::new();
        payloads.insert("signups".to_string(), json!({ "type": "signups" }));
        payloads.insert(
            "broken_query".to_string(),
            json!({ "error": "syntax error" }),
        );

        let provider = Box::new(DummyProvider {
            name: "core_report".to_string(),
            payloads,
        });

        let registry = Registry::new(vec![provider]);
        let bulk_fetch = BulkFetch {
            registry: &registry,
        };

        let items = vec![
            BulkFetchItem {
                source: "core_report".to_string(),
                identifier: "signups".to_string(),
            },
            BulkFetchItem {
                source: "core_report".to_string(),
                identifier: "broken_query".to_string(),
            },
            BulkFetchItem {
                source: "core_report".to_string(),
                identifier: "missing".to_string(),
            },
            BulkFetchItem {
                source: "unknown".to_string(),
                identifier: "x".to_string(),
            },
        ];

        let filters = HashMap::new();
        let result = bulk_fetch.call(&items, &filters, &());

        assert_eq!(result.items.len(), 4);
    }
}
