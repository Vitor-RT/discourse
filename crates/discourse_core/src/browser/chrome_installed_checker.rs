use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum ChromeError {
    ChromeNotInstalled(String),
    ChromeVersionError(String),
    ChromeVersionTooLow(String),
}

pub fn run() -> Result<String, ChromeError> {
    let mut binary = None;

    if Command::new("sh")
        .arg("-c")
        .arg("command -v google-chrome >/dev/null")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        binary = Some("google-chrome".to_string());
    }

    if binary.is_none()
        && Command::new("sh")
<<<<<<< HEAD
=======
            .arg("-c")
            .arg("command -v google-chrome >/dev/null")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    {
        binary = Some("google-chrome".to_string());
    }

    if binary.is_none()
        && Command::new("sh")
>>>>>>> origin/rust-migration-6812869939882800025
            .arg("-c")
            .arg("command -v chromium >/dev/null")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    {
        binary = Some("chromium".to_string());
    }

    let binary = binary.ok_or_else(|| {
        ChromeError::ChromeNotInstalled(
            "Chrome is not installed. Download from https://www.google.com/chrome/browser/desktop/index.html".to_string(),
        )
    })?;

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .map_err(|_| {
            ChromeError::ChromeVersionError(format!("Can't get the {} version", binary))
        })?;
    let version = String::from_utf8_lossy(&output.stdout);

    let re = regex::Regex::new(r"[\d\.]+").unwrap();
    let version_match = re.find(&version).ok_or_else(|| {
        ChromeError::ChromeVersionError(format!("Can't get the {} version", binary))
    })?;
    let version_str = version_match.as_str();

    let major_version: u32 = version_str
        .split('.')
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    if major_version < 59 {
        return Err(ChromeError::ChromeVersionTooLow(
            "Chrome 59 or higher is required".to_string(),
        ));
    }

    if binary.contains("chromium") {
        Ok("Chromium".to_string())
    } else {
        Ok("Chrome".to_string())
    }
}
