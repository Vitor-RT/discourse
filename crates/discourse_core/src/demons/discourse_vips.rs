use crate::demons::base::Demon;
use std::fs;
use std::path::Path;

pub struct DiscourseVips {
    index: u32,
    pid: Option<u32>,
    started: bool,
    rails_root: String,
    worker_process_active: bool,
}

impl DiscourseVips {
    pub fn new(index: u32, rails_root: String) -> Self {
        Self {
            index,
            pid: None,
            started: false,
            rails_root,
            worker_process_active: false,
        }
    }

    pub fn release_inherited_worker(&mut self) {
        self.worker_process_active = false;
    }
}

impl Demon for DiscourseVips {
    fn prefix() -> &'static str {
        "discourse_vips"
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
        self.worker_process_active = false;
        // mock worker process start
        self.worker_process_active = true;
        self.pid = Some(54321);
        self.write_pid_file(54321);
    }

    fn stop(&mut self) {
        let worker_pid = self.pid;
        self.started = false;
        self.worker_process_active = false;
        self.pid = None;

        if let Some(pid) = worker_pid {
            let p = self.pid_file();
            if Path::new(&p).exists() {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(file_pid) = content.trim().parse::<u32>() {
                        if file_pid == pid {
                            let _ = fs::remove_file(&p);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discourse_vips_lifecycle() {
        let mut d = DiscourseVips::new(1, "/tmp".to_string());

        // Start
        d.start();
        assert!(d.started());
        assert!(d.worker_process_active);
        assert_eq!(d.get_pid(), Some(54321));
        assert!(Path::new(&d.pid_file()).exists());

        // Release worker
        d.release_inherited_worker();
        assert!(!d.worker_process_active);

        // Stop
        d.stop();
        assert!(!d.started());
        assert!(!d.worker_process_active);
        assert_eq!(d.get_pid(), None);
        assert!(!Path::new(&d.pid_file()).exists());
    }
}
