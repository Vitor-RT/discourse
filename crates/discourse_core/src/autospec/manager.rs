use crate::autospec::simple_runner::SimpleRunner;
use std::sync::{Arc, Condvar, Mutex};
use std::collections::HashMap;
use std::thread;
use crate::autospec::base_runner::BaseRunner;
use notify::{Watcher, RecursiveMode, Result as NotifyResult};

pub struct Manager {
    pub debug: bool,
    pub auto_run_all: bool,
    pub queue: Arc<Mutex<Vec<(String, String)>>>,
    pub signal: Arc<Condvar>,
    pub simple_runner: Arc<Mutex<SimpleRunner>>,
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            debug: false,
            auto_run_all: true,
            queue: Arc::new(Mutex::new(Vec::new())),
            signal: Arc::new(Condvar::new()),
            simple_runner: Arc::new(Mutex::new(SimpleRunner::new())),
        }
    }

    pub fn run(&mut self) {
        println!("Rust manager started.");

        self.start_runners();
        self.start_service_queue();

        // Spawn file watcher
        let queue_clone = Arc::clone(&self.queue);
        let signal_clone = Arc::clone(&self.signal);
        let abort_runner = Arc::clone(&self.simple_runner);
        thread::spawn(move || {
            let mut watcher = notify::recommended_watcher(move |res: NotifyResult<notify::Event>| {
                match res {
                    Ok(event) => {
                        let mut changes = Vec::new();
                        for path in event.paths {
                            if let Some(p) = path.to_str() {
                                // Ignore lib/autospec explicitly as in ruby version
                                if p.contains("lib/autospec") {
                                    continue;
                                }
                                changes.push((p.to_string(), p.to_string()));
                            }
                        }

                        if !changes.is_empty() {
                            abort_runner.lock().unwrap().abort();
                            let mut q = queue_clone.lock().unwrap();
                            for (file, spec) in changes {
                                q.retain(|(_, s)| !s.trim().starts_with(spec.trim()));
                                q.insert(0, (file, spec));
                            }
                            signal_clone.notify_all();
                        }
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }).unwrap();

            // Watch relevant directories just like original code
            let dirs = vec!["spec", "lib", "app", "config", "test", "vendor", "plugins"];
            for dir in dirs {
                let _ = watcher.watch(std::path::Path::new(dir), RecursiveMode::Recursive);
            }

            loop {
                thread::sleep(std::time::Duration::from_secs(60));
            }
        });

        println!("Press [ENTER] to stop the current run");
        if !self.auto_run_all {
            println!("Press [ENTER] while stopped to run all specs");
        }

        self.process_queue();
    }

    pub fn ensure_all_specs_will_run(&mut self) {
        let mut q = self.queue.lock().unwrap();
        q.retain(|(_, s)| s != "spec");
        q.push(("spec".to_string(), "spec".to_string()));
    }

    pub fn start_runners(&self) {
        // SimpleRunner doesn't require explicit start
    }

    pub fn stop_runners(&self) {
        let runner = self.simple_runner.lock().unwrap();
        runner.stop();
    }

    pub fn abort_runners(&mut self) {
        let runner = self.simple_runner.lock().unwrap();
        runner.abort();
    }

    pub fn start_service_queue(&self) {
        let queue = Arc::clone(&self.queue);
        let signal = Arc::clone(&self.signal);
        let runner = Arc::clone(&self.simple_runner);
        let auto_run_all = self.auto_run_all;

        thread::spawn(move || {
            loop {
                let mut current: Option<(String, String)> = None;
                {
                    let q = queue.lock().unwrap();
                    if let Some(c) = q.first() {
                        current = Some(c.clone());
                    }
                }

                let mut last_failed = false;
                if let Some(c) = current {
                    last_failed = {
                        let result = {
                            let r = runner.lock().unwrap();
                            r.run(&c.1)
                        };

                        if result == 0 {
                            let mut q = queue.lock().unwrap();
                            if !q.is_empty() {
                                q.remove(0);
                            }
                            false
                        } else {
                            true
                        }
                    };

                    if last_failed {
                        let mut q = queue.lock().unwrap();
                        if let Some(first) = q.first() {
                            if first.0 == "focus" {
                                q.remove(0);
                            }
                        }

                        let failed_specs = {
                            let r = runner.lock().unwrap();
                            r.failed_specs()
                        };

                        let specs_to_focus = failed_specs.into_iter().take(11).collect::<Vec<_>>();

                        if !specs_to_focus.is_empty() {
                            q.insert(0, ("focus".to_string(), specs_to_focus.join(" ")));
                        }

                        if auto_run_all {
                            q.retain(|(_, s)| s != "spec");
                            q.push(("spec".to_string(), "spec".to_string()));
                        }
                    }
                }

                let mut guard = queue.lock().unwrap();
                if guard.is_empty() || last_failed {
                    guard = signal.wait(guard).unwrap();
                }
            }
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

    pub fn listen_for_changes(&self) {
        // Covered in run via thread
    }

    pub fn process_change(&mut self, _files: Vec<(String, Option<usize>)>) {
    }

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
            q.retain(|(_, s)| !s.trim().starts_with(spec.trim()));

            if let Some(focus) = q.first().cloned() {
                if focus.0 == "focus" {
                    q.remove(0);
                    q.insert(0, (file.clone(), spec.clone()));

                    if !spec.contains(':') || !focus.1.contains(spec.split(':').next().unwrap()) {
                        if focus.1.contains(&spec) || file != spec {
                            q.insert(0, focus.clone());
                        }
                    }
                } else {
                    q.insert(0, (file.clone(), spec.clone()));
                }
            } else {
                q.insert(0, (file.clone(), spec.clone()));
            }

            if self.auto_run_all {
                q.retain(|(_, s)| s != "spec");
                q.push(("spec".to_string(), "spec".to_string()));
            }
        }

        self.signal.notify_all();
    }

    pub fn process_queue(&mut self) {
        let is_empty = {
            let q = self.queue.lock().unwrap();
            q.is_empty()
        };

        if is_empty {
            self.ensure_all_specs_will_run();
            self.signal.notify_all();
        } else {
            let specs = {
                let r = self.simple_runner.lock().unwrap();
                r.failed_specs()
            };
            println!("\n\n");
            if specs.is_empty() {
                println!("No specs have failed yet! Aborting anyway\n");
                self.abort_runners();
            } else {
                println!("The following specs have failed:");
                for s in &specs {
                    println!("{}", s);
                }
                println!();
                let new_specs: Vec<(String, String)> = specs.into_iter().map(|s| (s.clone(), s)).collect();
                self.queue_specs(new_specs);
            }
        }
    }

    pub fn fail(&self, message: &str) {
        println!("Failed: {}", message);
    }
}
