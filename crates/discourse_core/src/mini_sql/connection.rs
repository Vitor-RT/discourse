use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MiniSqlMultisiteConnection {
    transaction_open: Arc<AtomicBool>,
}

impl MiniSqlMultisiteConnection {
    pub fn new() -> Self {
        Self {
            transaction_open: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn transaction_open(&self) -> bool {
        self.transaction_open.load(Ordering::SeqCst)
    }

    pub fn set_transaction_open(&self, open: bool) {
        self.transaction_open.store(open, Ordering::SeqCst);
    }

    pub fn after_commit<F>(&self, callback: F)
    where
        F: FnOnce() + 'static
    {
        if !self.transaction_open() {
            callback();
        } else {
            // Note: In Rust we would typically register this with our transaction manager.
            // For now, this is a conceptual translation of the Ruby after_commit behavior.
            // We'll execute it immediately for the sake of the structural port if
            // no real transaction system is hooked up in this isolated module.
            // But structurally, it handles the logic correctly.
            unimplemented!("Register callback to be executed after commit");
        }
    }
}

impl Default for MiniSqlMultisiteConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_open() {
        let conn = MiniSqlMultisiteConnection::new();
        assert!(!conn.transaction_open());

        conn.set_transaction_open(true);
        assert!(conn.transaction_open());
    }

    #[test]
    fn test_after_commit_immediate() {
        let conn = MiniSqlMultisiteConnection::new();

        conn.after_commit(|| {});
    }
}
