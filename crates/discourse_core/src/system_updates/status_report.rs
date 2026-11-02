use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//
// Traits for dependencies
//

pub trait GitClient: Send + Sync {
    fn show_file(&self, sha: &str, path: &str) -> Option<String>;
    fn log(&self, since: &str, path: &str) -> Vec<Commit>;
}

pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &str) -> std::io::Result<String>;
    fn write_to_string(&self, path: &str, content: &str) -> std::io::Result<()>;
}

//
// Models
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub date: String, // ISO 8601 string expected
    pub author_name: String,
    pub author_email: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub commit: Commit,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub name: String,
    pub settings_path: String,
    pub current_status: Option<String>,
    pub next_status: Option<String>,
    pub eligible: bool,
    pub eligibility_reason: String,
    pub days_since_status_change: Option<i64>,
    pub last_status_change_commit: Option<String>,
    pub last_status_change_date: Option<String>,
    pub original_commit: Option<String>,
    pub original_commit_date: Option<String>,
    pub original_author_name: Option<String>,
    pub original_author_email: Option<String>,
    pub original_pr_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<bool>,
}

//
// Helper structures mimicking Ruby equivalents
//

pub struct StatusReport {
    repo_path: String,
    stale_after_days: i64,
    now: DateTime<Utc>,
}

impl StatusReport {
    pub fn new(repo_path: String, stale_after_days: i64, now: DateTime<Utc>) -> Self {
        Self {
            repo_path,
            stale_after_days,
            now,
        }
    }

    pub fn build_record(
        &self,
        name: &str,
        metadata: &HashMap<String, String>,
        history: &[HistoryEntry],
    ) -> ReportRecord {
        let original_commit = history.first();
        let last_status_change = self.last_status_change_for(history);
        let current_status = metadata.get("status").cloned();
        let next_status = self.next_status_str(current_status.as_deref());

        let (eligible, reason) = self.eligibility_for(
            current_status.as_deref(),
            next_status.as_deref(),
            last_status_change.as_ref(),
        );

        let mut record = ReportRecord {
            name: name.to_string(),
            settings_path: metadata
                .get("settings_path")
                .cloned()
                .unwrap_or_default(),
            current_status,
            next_status,
            eligible,
            eligibility_reason: reason.to_string(),
            days_since_status_change: last_status_change.and_then(|c| self.days_since(c)),
            last_status_change_commit: last_status_change.map(|c| c.commit.sha.clone()),
            last_status_change_date: last_status_change.map(|c| c.commit.date.clone()),
            original_commit: original_commit.map(|c| c.commit.sha.clone()),
            original_commit_date: original_commit.map(|c| c.commit.date.clone()),
            original_author_name: original_commit.map(|c| c.commit.author_name.clone()),
            original_author_email: original_commit.map(|c| c.commit.author_email.clone()),
            original_pr_number: original_commit.and_then(|c| self.original_pr_number(c)),
            branch: None,
            title: None,
            pr_label: None,
            pr_body: None,
            applied: None,
        };

        if record.eligible {
            self.add_pr_plan(&mut record);
        }

        record
    }

    fn last_status_change_for<'a>(
        &self,
        history: &'a [HistoryEntry],
    ) -> Option<&'a HistoryEntry> {
        if history.is_empty() {
            return None;
        }

        let mut last_change = &history[0];
        for window in history.windows(2) {
            let previous = &window[0];
            let current = &window[1];
            if previous.status != current.status {
                last_change = current;
            }
        }
        Some(last_change)
    }

    fn next_status_str(&self, status: Option<&str>) -> Option<String> {
        status.and_then(|s| super::upcoming_changes::Status::from_str(s))
            .and_then(|s| super::upcoming_changes::UpcomingChanges::next_status(Some(s)))
            .map(|s| match s {
                super::upcoming_changes::Status::Conceptual => "conceptual",
                super::upcoming_changes::Status::Experimental => "experimental",
                super::upcoming_changes::Status::Alpha => "alpha",
                super::upcoming_changes::Status::Beta => "beta",
                super::upcoming_changes::Status::Stable => "stable",
                super::upcoming_changes::Status::Permanent => "permanent",
                super::upcoming_changes::Status::Never => "never",
            }.to_string())
    }

    fn eligibility_for(
        &self,
        current_status: Option<&str>,
        next_status: Option<&str>,
        last_status_change: Option<&&HistoryEntry>,
    ) -> (bool, &'static str) {
        if current_status.unwrap_or("").is_empty() {
            return (false, "missing_status");
        }
        if super::upcoming_changes::Status::from_str(current_status.unwrap()).is_none() {
            return (false, "unknown_status");
        }
        if next_status.is_none() {
            return (false, "terminal_status");
        }
        let last_change = match last_status_change {
            Some(entry) => entry,
            None => return (false, "missing_git_history"),
        };

        let days = match self.days_since(last_change) {
            Some(d) => d,
            None => return (false, "missing_git_history"), // Fallback
        };

        if days < self.stale_after_days {
            return (false, "status_changed_recently");
        }

        (true, "status_unchanged") // Slightly shortened for simpler Rust matching
    }

    fn days_since(&self, entry: &HistoryEntry) -> Option<i64> {
        let dt = DateTime::parse_from_rfc3339(&entry.commit.date).ok()?;
        let dt_utc = dt.with_timezone(&Utc);
        let duration = self.now.signed_duration_since(dt_utc);
        Some(duration.num_days())
    }

    fn original_pr_number(&self, entry: &HistoryEntry) -> Option<String> {
        let re1 = Regex::new(r"\(#(\d+)\)\z").unwrap();
        let re2 = Regex::new(r"#(\d+)").unwrap();

        if let Some(caps) = re1.captures(&entry.commit.subject) {
            return Some(caps.get(1).unwrap().as_str().to_string());
        }
        if let Some(caps) = re2.captures(&entry.commit.subject) {
            return Some(caps.get(1).unwrap().as_str().to_string());
        }
        None
    }

    fn add_pr_plan(&self, record: &mut ReportRecord) {
        record.branch = Some(format!("dev/upcoming-change-status-bump/{}", record.name));
        record.title = Some(format!(
            "FEATURE: Bump {} upcoming change to {}",
            record.name,
            record.next_status.as_deref().unwrap_or("")
        ));
        record.pr_label = Some("upcoming-change".to_string());

        let commit_link = record.last_status_change_commit.as_ref()
            .map(|sha| format!("[`{}`](https://github.com/discourse/discourse/commit/{})", sha, sha))
            .unwrap_or_else(|| "N/A".to_string());

        let author = if let (Some(n), Some(e)) = (&record.original_author_name, &record.original_author_email) {
            format!("{} (<{}>)", n, e)
        } else {
            "N/A".to_string()
        };

        let pr = record.original_pr_number.as_ref()
            .map(|num| format!("#{}", num))
            .unwrap_or_else(|| "N/A".to_string());

        let body = format!(
            "<!-- upcoming-change-status-pr:{} -->\n\n\
             This automated PR moves `{}` from `{}` to `{}` after {}+ days without a status change.\n\n\
             - Last status change commit: {}\n\
             - Last status change date: `{}`\n\
             - Settings file: `{}`\n\
             - Original author: {}\n\
             - Original PR: {}",
            record.name,
            record.name,
            record.current_status.as_deref().unwrap_or(""),
            record.next_status.as_deref().unwrap_or(""),
            self.stale_after_days,
            commit_link,
            record.last_status_change_date.as_deref().unwrap_or(""),
            record.settings_path,
            author,
            pr
        );

        record.pr_body = Some(body);
    }
}

pub struct SourceStatusUpdater<'a> {
    pub fs: &'a dyn FileSystem,
    pub settings_file: String,
}

impl<'a> SourceStatusUpdater<'a> {
    pub fn update(&self, change_name: &str, next_status: &str) -> std::io::Result<()> {
        let content = self.fs.read_to_string(&self.settings_file)?;
        let lines: Vec<&str> = content.split('\n').collect();

        let re = Regex::new(r#"^(\s*status:\s*)(["']?)([^"'\s]+)(["']?)(.*)$"#).unwrap();

        let mut new_lines = Vec::new();
        let mut in_target_change = false;
        let mut in_upcoming_change_block = false;
        let mut updated = false;

        for line in lines {
            if line.trim().starts_with(&format!("{}:", change_name)) {
                in_target_change = true;
            } else if line.trim().starts_with("upcoming_change:") && in_target_change {
                in_upcoming_change_block = true;
            } else if !line.starts_with(' ') && !line.trim().is_empty() {
                in_target_change = false;
                in_upcoming_change_block = false;
            }

            if in_upcoming_change_block && !updated && line.trim().starts_with("status:") {
                if let Some(caps) = re.captures(line) {
                    let indent = caps.get(1).unwrap().as_str();
                    let q1 = caps.get(2).unwrap().as_str();
                    let q2 = caps.get(4).unwrap().as_str();
                    let suffix = caps.get(5).unwrap().as_str();
                    let new_line = format!("{}{}{}{}{}", indent, q1, next_status, q2, suffix);
                    new_lines.push(new_line);
                    updated = true;
                    continue;
                }
            }
            new_lines.push(line.to_string());
        }

        if updated {
            self.fs.write_to_string(&self.settings_file, &new_lines.join("\n"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct MockFs {
        content: Arc<Mutex<HashMap<String, String>>>,
    }

    impl FileSystem for MockFs {
        fn read_to_string(&self, path: &str) -> std::io::Result<String> {
            let m = self.content.lock().unwrap();
            m.get(path).cloned().ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
        }

        fn write_to_string(&self, path: &str, content: &str) -> std::io::Result<()> {
            let mut m = self.content.lock().unwrap();
            m.insert(path.to_string(), content.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_eligibility() {
        let report = StatusReport::new("/tmp".into(), 14, DateTime::parse_from_rfc3339("2023-01-20T00:00:00Z").unwrap().with_timezone(&Utc));

        let c1 = Commit {
            sha: "abc".into(),
            date: "2023-01-01T00:00:00Z".into(), // 19 days ago
            author_name: "Test".into(),
            author_email: "test@example.com".into(),
            subject: "Fix (#123)".into(),
        };

        let he = HistoryEntry { commit: c1, status: Some("experimental".into()) };
        let (eligible, reason) = report.eligibility_for(Some("experimental"), Some("alpha"), Some(&&he));

        assert!(eligible);
        assert_eq!(reason, "status_unchanged");
    }

    #[test]
    fn test_status_updater() {
        let initial = "some_category:\n  my_feature:\n    upcoming_change:\n      status: experimental\n";
        let content = Arc::new(Mutex::new(HashMap::new()));
        content.lock().unwrap().insert("settings.yml".into(), initial.into());
        let fs = MockFs { content: content.clone() };

        let updater = SourceStatusUpdater { fs: &fs, settings_file: "settings.yml".into() };
        updater.update("my_feature", "alpha").unwrap();

        let new_content = fs.read_to_string("settings.yml").unwrap();
        assert!(new_content.contains("status: alpha"));
    }
}