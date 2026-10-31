use crate::primitive::{Kind, Primitive};
use crate::registry::Registry;
use std::collections::HashSet;

pub struct RequestContext {
    pub scopes: HashSet<String>,
}

impl RequestContext {
    pub fn has_scopes(&self, required_scopes: &[String]) -> bool {
        required_scopes
            .iter()
            .all(|scope| self.scopes.contains(scope))
    }
}

pub struct Policies {
    pub exposed_identifiers: HashSet<String>,
}

pub struct Catalog<'a> {
    request_context: RequestContext,
    registry: &'a Registry,
    policies: &'a Policies,
}

impl<'a> Catalog<'a> {
    pub fn new(
        request_context: RequestContext,
        registry: &'a Registry,
        policies: &'a Policies,
    ) -> Self {
        Self {
            request_context,
            registry,
            policies,
        }
    }

    pub fn list(&self, kind: Kind) -> Vec<Primitive> {
        self.exposed(kind)
            .into_iter()
            .filter(|primitive| self.request_context.has_scopes(&primitive.required_scopes))
            .collect()
    }

    pub fn find(&self, kind: Kind, identifier: &str) -> Option<Primitive> {
        self.list(kind)
            .into_iter()
            .find(|primitive| primitive.identifier == identifier)
    }

    pub fn find_exposed(&self, kind: Kind, identifier: &str) -> Option<Primitive> {
        self.exposed(kind)
            .into_iter()
            .find(|primitive| primitive.identifier == identifier)
    }

    fn exposed(&self, kind: Kind) -> Vec<Primitive> {
        self.registry
            .all(Some(kind))
            .into_iter()
            .filter(|primitive| {
                self.policies
                    .exposed_identifiers
                    .contains(&primitive.identifier)
                    && primitive.available()
            })
            .collect()
    }
}
