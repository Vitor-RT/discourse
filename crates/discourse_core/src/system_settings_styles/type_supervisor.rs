pub struct TypeSupervisor {
    types: std::collections::HashMap<String, i32>,
}

impl TypeSupervisor {
    pub fn new() -> Self {
        Self {
            types: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_supervisor_initialization() {
        let supervisor = TypeSupervisor::new();
        assert_eq!(supervisor.types.len(), 0);
    }
}
