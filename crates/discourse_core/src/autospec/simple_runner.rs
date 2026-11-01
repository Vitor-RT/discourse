use crate::autospec::base_runner::BaseRunner;
use crate::autospec::formatter::Formatter;
use crate::autospec::rspec_runner::RspecRunner;
use std::env;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SimpleRunner {
    pub rspec_runner: RspecRunner,
    pub mutex: Arc<Mutex<Option<Child>>>,
}

impl Default for SimpleRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleRunner {
    pub fn new() -> Self {
        SimpleRunner {
            rspec_runner: RspecRunner::new(),
            mutex: Arc::new(Mutex::new(None)),
        }
    }

    pub fn wait_for_done(&self, _pid: u32) {
        thread::sleep(Duration::from_millis(100));
    }

    pub fn failed_specs(&self) -> Vec<String> {
        self.rspec_runner.failed_specs()
    }
}

impl BaseRunner for SimpleRunner {
    fn run(&self, specs: &str) -> i32 {
        println!("Running Rspec: {}", specs);
        self.abort();

        let formatter_path = Formatter::setup_ruby_formatter();
        let args = vec![
            "-r".to_string(),
            formatter_path,
            "-f".to_string(),
            "Autospec::Formatter".to_string(),
        ];

        let line_specified = specs.split_whitespace().any(|s| s.contains(':'));
        let multiple_files = specs.split_whitespace().count() > 1 || specs == "spec";

        let command_str = if env::var("PARALLEL_SPEC").unwrap_or_default() == "1"
            && multiple_files
            && !line_specified
        {
            "bin/turbo_rspec"
        } else {
            "bin/rspec"
        };

        let mut cmd = Command::new(command_str);
        for arg in &args {
            cmd.arg(arg);
        }
        for spec in specs.split_whitespace() {
            cmd.arg(spec);
        }

        cmd.env("RAILS_ENV", "test");
        if specs
            .split_whitespace()
            .any(|s| s.starts_with("plugins") || s.starts_with("./plugins"))
        {
            cmd.env("LOAD_PLUGINS", "1");
            println!("Loading plugins while running specs");
        }

        let child_process = cmd.spawn();

        match child_process {
            Ok(child) => {
                {
                    let mut child_guard = self.mutex.lock().unwrap();
                    *child_guard = Some(child);
                }

                let mut status_code = 1;
                let mut finished = false;
                while !finished {
                    let mut child_guard = self.mutex.lock().unwrap();
                    if let Some(child_ref) = child_guard.as_mut() {
                        match child_ref.try_wait() {
                            Ok(Some(status)) => {
                                status_code = status.code().unwrap_or(1);
                                finished = true;
                            }
                            Ok(None) => {}
                            Err(_) => {
                                finished = true;
                            }
                        }
                    } else {
                        finished = true;
                    }
                    drop(child_guard);
                    if !finished {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
                status_code
            }
            Err(e) => {
                eprintln!("Failed to execute process: {}", e);
                1
            }
        }
    }

    fn abort(&self) {
        if let Ok(mut child_opt) = self.mutex.lock() {
            if let Some(child) = child_opt.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
            *child_opt = None;
        }
    }

    fn stop(&self) {
        self.abort();
    }
}
