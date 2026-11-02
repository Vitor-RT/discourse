use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use super::base_runner::BaseRunner;
use super::simple_runner::SimpleRunner;

pub struct Manager {
    pub queue: Arc<Mutex<Vec<(String, String)>>>,
    pub signal: Arc<Condvar>,
    pub simple_runner: Arc<Mutex<SimpleRunner>>,
    pub auto_run_all: bool,
}

impl Manager {
    pub fn new() -> Self {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let signal = Arc::new(Condvar::new());
        let simple_runner = Arc::new(Mutex::new(SimpleRunner::new()));

        Self {
            queue,
            signal,
            simple_runner,
            auto_run_all: true,
        }
    }

    pub fn run(&mut self) {
        self.start_runners();
        self.start_service_queue();
    }

    pub fn start_runners(&self) {
        let mut runner = self.simple_runner.lock().unwrap();
        runner.start();
    }

    pub fn stop_runners(&self) {
        let runner = self.simple_runner.lock().unwrap();
        runner.stop();
    }

    pub fn abort_runners(&self) {
        let runner = self.simple_runner.lock().unwrap();
        runner.abort();
    }

    pub fn start_service_queue(&self) {
        let queue = self.queue.clone();
        let signal = self.signal.clone();
        let runner_arc = self.simple_runner.clone();

        thread::spawn(move || loop {
            let (file, spec) = {
                let mut q = queue.lock().unwrap();
                while q.is_empty() {
                    q = signal.wait(q).unwrap();
                }
                q.remove(0)
            };

            let runner = runner_arc.lock().unwrap();
            runner.run(&format!("{} {}", file, spec));
        });
    }

    pub fn root_path(&self) -> String {
        ".".to_string()
    }

    pub fn reverse_symlink_map(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    pub fn reverse_symlink(&self, file: &str) -> String {
        file.to_string()
    }

    pub fn listen_for_changes(&self) {}

    pub fn process_change(&mut self, _files: Vec<(String, Option<usize>)>) {}

    pub fn queue_specs(&mut self, specs: Vec<(String, String)>) {
        if specs.is_empty() {
            if let Ok(_guard) = self.queue.try_lock() {
                self.signal.notify_all();
            }
            return;
        } else {
            self.abort_runners();
        }

        let mut q = self.queue.lock().unwrap();
        for (file, spec) in specs {
            q.push((file, spec));
        }
        self.signal.notify_all();
    }

    pub fn process_queue(&mut self) {
        let is_empty = {
            let q = self.queue.lock().unwrap();
            q.is_empty()
        };

        if is_empty {
            self.signal.notify_all();
        } else {
            let specs = {
                let r = self.simple_runner.lock().unwrap();
                r.failed_specs()
            };
            if specs.is_empty() {
                self.abort_runners();
            } else {
                let new_specs: Vec<(String, String)> = specs.into_iter().map(|s| (s.clone(), s)).collect();
                self.queue_specs(new_specs);
            }
        }
    }

    pub fn fail(&self, message: &str) {
        println!("Failed: {}", message);
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
