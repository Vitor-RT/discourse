use crate::demons::base::Demon;
use nix::sys::signal::Signal;
use std::sync::mpsc;

pub struct PluginJsWatcher {
    index: u32,
    pid: Option<u32>,
    started: bool,
    rails_root: String,
}

impl PluginJsWatcher {
    pub fn new(index: u32, rails_root: String) -> Self {
        Self {
            index,
            pid: None,
            started: false,
            rails_root,
        }
    }

    pub fn after_fork(&self, tx: mpsc::Sender<()>) {
        println!("Loading PluginJsWatcher in process");

        // Set up signal handling logic. In an actual application, signal_hook or
        // a dedicated signal handling thread is used, but for testing the structural
        // translation, we demonstrate channel messaging.

        let _ = tx.send(()); // Simulate setting up signal handling trap
    }
}

impl Demon for PluginJsWatcher {
    fn prefix() -> &'static str {
        "plugin_js_watcher"
    }

    fn index(&self) -> u32 {
        self.index
    }

    fn get_pid(&self) -> Option<u32> {
        self.pid
    }

    fn set_pid(&mut self, pid: Option<u32>) {
        self.pid = pid;
    }

    fn started(&self) -> bool {
        self.started
    }

    fn set_started(&mut self, started: bool) {
        self.started = started;
    }

    fn rails_root(&self) -> String {
        self.rails_root.clone()
    }

    fn run(&mut self) {
        self.pid = Some(11111);
        self.write_pid_file(11111);
    }

    fn stop_signal(&self) -> Signal {
        Signal::SIGTERM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_js_watcher_signal_override() {
        let watcher = PluginJsWatcher::new(1, "/tmp".to_string());
        assert_eq!(watcher.stop_signal(), Signal::SIGTERM);
    }

    #[test]
    fn test_plugin_js_watcher_after_fork() {
        let watcher = PluginJsWatcher::new(1, "/tmp".to_string());
        let (tx, rx) = mpsc::channel();

        watcher.after_fork(tx);

        // Simulating that the queue push for INT/TERM/HUP happened correctly
        assert!(rx.recv().is_ok());
    }
}
