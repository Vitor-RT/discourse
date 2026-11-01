use crate::admin_reports::registry::Registry;
use serde_json::Value;
use std::collections::HashMap;

pub struct BulkFetchItem {
    pub source: String,
    pub identifier: String,
}

pub struct BulkFetchResultItem {
    pub source: String,
    pub identifier: String,
    pub key: String,
    pub data: Option<Value>,
    pub error: bool,
}

pub struct BulkFetchResult {
    pub items: Vec<BulkFetchResultItem>,
}

pub struct BulkFetch<'a> {
    pub registry: &'a Registry,
}

impl<'a> BulkFetch<'a> {
    pub fn call(
        &self,
        items: &[BulkFetchItem],
        filters: &HashMap<String, String>,
        guardian: &dyn std::any::Any,
    ) -> BulkFetchResult {
        let payloads: Vec<Result<Option<Value>, ()>> = items
            .iter()
            .map(|item| self.fetch_item(item, filters, guardian))
            .collect();

        let mut results = Vec::with_capacity(items.len());
        for (item, payload_res) in items.iter().zip(payloads) {
            let key = format!("{}:{}", item.source, item.identifier);
            match payload_res {
                Ok(Some(data)) => {
                    let mut errored = false;
                    if let Some(obj) = data.as_object() {
                        if obj.get("error").is_some() {
                            errored = true;
                        }
                    }
                    results.push(BulkFetchResultItem {
                        source: item.source.clone(),
                        identifier: item.identifier.clone(),
                        key,
                        data: Some(data),
                        error: errored,
                    });
                }
                Ok(None) => {
                    results.push(BulkFetchResultItem {
                        source: item.source.clone(),
                        identifier: item.identifier.clone(),
                        key,
                        data: None,
                        error: false,
                    });
                }
                Err(_) => {
                    results.push(BulkFetchResultItem {
                        source: item.source.clone(),
                        identifier: item.identifier.clone(),
                        key,
                        data: None,
                        error: true,
                    });
                }
            }
        }

        BulkFetchResult { items: results }
    }

    fn fetch_item(
        &self,
        item: &BulkFetchItem,
        filters: &HashMap<String, String>,
        guardian: &dyn std::any::Any,
    ) -> Result<Option<Value>, ()> {
        let provider = match self.registry.provider_for(&item.source) {
            Some(p) => p,
            None => return Ok(None),
        };

        let identifiers = vec![item.identifier.clone()];
        let mut result_map = provider.fetch_many(&identifiers, guardian, filters);

        Ok(result_map.remove(&item.identifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_reports::resolved_report::ResolvedReport;
    use crate::admin_reports::source_provider::SourceProvider;
    use serde_json::json;

    struct DummyProvider {
        name: String,
        payloads: HashMap<String, Value>,
    }

    impl SourceProvider for DummyProvider {
        fn source_name(&self) -> String {
            self.name.clone()
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
            ids: &[String],
            _g: &dyn std::any::Any,
            _f: &HashMap<String, String>,
        ) -> HashMap<String, Value> {
            let mut result = HashMap::new();
            for id in ids {
                if let Some(v) = self.payloads.get(id) {
                    result.insert(id.clone(), v.clone());
                }
            }
            result
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

        assert_eq!(result.items[0].identifier, "signups");
        assert_eq!(result.items[0].error, false);
        assert_eq!(result.items[0].data, Some(json!({ "type": "signups" })));

        assert_eq!(result.items[1].identifier, "broken_query");
        assert_eq!(result.items[1].error, true);
        assert_eq!(
            result.items[1].data,
            Some(json!({ "error": "syntax error" }))
        );

        assert_eq!(result.items[2].identifier, "missing");
        assert_eq!(result.items[2].error, false);
        assert_eq!(result.items[2].data, None);

        assert_eq!(result.items[3].identifier, "x");
        assert_eq!(result.items[3].error, false);
        assert_eq!(result.items[3].data, None);
    }
}
