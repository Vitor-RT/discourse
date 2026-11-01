use std::path::Path;
use std::process::Command;

pub const DEFAULT_READ_PATHS: &[&str] = &["/bin", "/lib", "/lib64", "/usr"];
pub const DEFAULT_EXECUTE_PATHS: &[&str] = &["/bin", "/lib", "/lib64", "/usr"];

pub struct SafeExec;

#[derive(Debug)]
pub struct CommandError {
    pub message: String,
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

impl SafeExec {
    pub fn capture(
        command: &[&str],
        _read: &[&str],
        _write: &[&str],
        _execute: &[&str],
        failure_message: &str,
        success_status_codes: &[i32],
    ) -> Result<String, CommandError> {
        if command.is_empty() {
            return Err(CommandError {
                message: "Empty command".to_string(),
                stdout: "".to_string(),
                stderr: "No command provided".to_string(),
                status: -1,
            });
        }

        let output = Command::new(command[0])
            .args(&command[1..])
            .output()
            .map_err(|e| CommandError {
                message: format!("Failed to execute command: {}", e),
                stdout: "".to_string(),
                stderr: e.to_string(),
                status: -1,
            })?;

        let status = output.status.code().unwrap_or(-1);

        if !success_status_codes.contains(&status) {
            let mut message = command.join(" ");
            if !failure_message.is_empty() {
<<<<<<< HEAD
                message.push('\n');
=======
                message.push_str("\n");
>>>>>>> origin/rust-migration-7847495834899956704
                message.push_str(failure_message);
            }
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !stderr.is_empty() {
<<<<<<< HEAD
                message.push('\n');
=======
                message.push_str("\n");
>>>>>>> origin/rust-migration-7847495834899956704
                message.push_str(&stderr);
            }

            return Err(CommandError {
                message,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr,
                status,
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn landlock_supported() -> bool {
        // Mock landlock support checking
        false
    }

    pub fn existing_paths(paths: &[&str]) -> Vec<String> {
        paths
            .iter()
            .filter(|&&path| !path.is_empty() && Path::new(path).exists())
            .map(|&s| s.to_string())
            .collect()
    }

    pub fn default_read_paths() -> Vec<String> {
        Self::existing_paths(DEFAULT_READ_PATHS)
    }

    pub fn default_execute_paths() -> Vec<String> {
        Self::existing_paths(DEFAULT_EXECUTE_PATHS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_success() {
        let result = SafeExec::capture(&["echo", "hello"], &[], &[], &[], "", &[0]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_capture_failure() {
        let result = SafeExec::capture(
            &["ls", "/does/not/exist/ever"],
            &[],
            &[],
            &[],
            "Custom Failure Message",
            &[0],
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("ls /does/not/exist/ever"));
        assert!(err.message.contains("Custom Failure Message"));
        assert!(err.status != 0);
    }

    #[test]
    fn test_existing_paths() {
        let paths = SafeExec::existing_paths(&["/bin", "/does_not_exist"]);
        // Assume /bin exists on test environment
        assert!(paths.contains(&"/bin".to_string()));
        assert!(!paths.contains(&"/does_not_exist".to_string()));
    }
}
