use nix::sys::signal::{self, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag};
use nix::unistd::Pid;
use std::fs::{self, File};
use std::io::Write;
use std::time::Duration;

pub trait Demon {
    fn prefix() -> &'static str;

    fn index(&self) -> u32;

    fn get_pid(&self) -> Option<u32>;
    fn set_pid(&mut self, pid: Option<u32>);

    fn started(&self) -> bool;
    fn set_started(&mut self, started: bool);

    fn rails_root(&self) -> String;

    fn pid_file(&self) -> String {
        format!(
            "{}/tmp/pids/{}_{}.pid",
            self.rails_root(),
            Self::prefix(),
            self.index()
        )
    }

    fn write_pid_file(&self, pid: u32) {
        let p = self.pid_file();
        let path = std::path::Path::new(&p);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = File::create(path) {
            let _ = write!(file, "{}", pid);
        }
    }

    fn delete_pid_file(&self) {
        let _ = fs::remove_file(self.pid_file());
    }

    fn stop_signal(&self) -> Signal {
        Signal::SIGHUP
    }

    fn stop_timeout(&self) -> u32 {
        10
    }

    fn already_running(&self) -> Option<u32> {
        if let Ok(content) = fs::read_to_string(self.pid_file()) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if alive(pid) {
                    return Some(pid);
                }
            }
        }
        None
    }

    fn start(&mut self) {
        if self.get_pid().is_some() || self.started() {
            return;
        }

        if let Some(existing) = self.already_running() {
            let _ = signal::kill(Pid::from_raw(existing as i32), Signal::SIGTERM);
        }

        self.set_started(true);
        self.run();
    }

    fn run(&mut self);

    fn stop(&mut self) {
        self.set_started(false);
        if let Some(pid) = self.get_pid() {
            let stop_sig = self.stop_signal();
            let _ = signal::kill(Pid::from_raw(pid as i32), stop_sig);

            let mut timeout = self.stop_timeout() as f32;
            let step = (self.stop_timeout() as f32) / 10.0;

            while alive(pid) && timeout > 0.0 {
                timeout -= step;
                std::thread::sleep(Duration::from_secs_f32(step));
                let _ = waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG));
            }

            let _ = waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG));

            if alive(pid) {
                let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
            }

            let _ = waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG));

            self.set_pid(None);
            self.set_started(false);
        }
    }

    fn restart(&mut self) {
        self.stop();
        self.start();
    }

    fn ensure_running(&mut self) {
        if !self.started() {
            return;
        }

        if self.get_pid().is_none() {
            self.set_started(false);
            self.start();
            return;
        }

        if let Some(pid) = self.get_pid() {
            let dead = match waitpid(Pid::from_raw(pid as i32), Some(WaitPidFlag::WNOHANG)) {
                Ok(nix::sys::wait::WaitStatus::StillAlive) => false,
                Ok(_) => true,
                Err(_) => true,
            };

            if dead {
                self.set_pid(None);
                self.set_started(false);
                self.start();
            }
        }
    }

    fn kill(&mut self, sig: Signal) {
        if let Some(pid) = self.get_pid() {
            let _ = signal::kill(Pid::from_raw(pid as i32), sig);
        }
    }
}

pub fn alive(pid: u32) -> bool {
    signal::kill(Pid::from_raw(pid as i32), None).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDemon {
        index: u32,
        pid: Option<u32>,
        started: bool,
    }

    impl TestDemon {
        fn new(index: u32) -> Self {
            Self {
                index,
                pid: None,
                started: false,
            }
        }
    }

    impl Demon for TestDemon {
        fn prefix() -> &'static str {
            "test_demon"
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
            "/tmp".to_string()
        }
        fn run(&mut self) {
            self.pid = Some(12345);
            self.write_pid_file(12345);
        }
    }

    #[test]
    fn test_pid_file() {
        let d = TestDemon::new(1);
        assert_eq!(d.pid_file(), "/tmp/tmp/pids/test_demon_1.pid");
    }

    #[test]
    fn test_start() {
        let mut d = TestDemon::new(1);
        d.start();
        assert!(d.started());
        assert_eq!(d.get_pid(), Some(12345));
    }

    #[test]
    fn test_stop() {
        let mut d = TestDemon::new(1);
        d.start();
        d.stop();
        assert!(!d.started());
        assert_eq!(d.get_pid(), None);
    }

    #[test]
    fn test_ensure_running_when_stopped() {
        let mut d = TestDemon::new(1);
        d.ensure_running();
        assert!(!d.started());
    }
}
