pub trait Poller {
    fn poll_mailbox<F>(&self, process_cb: F) -> Result<(), String>
    where
        F: FnMut(&str) -> Result<(), String>;

    fn enabled(&self) -> bool {
        true
    }
}
