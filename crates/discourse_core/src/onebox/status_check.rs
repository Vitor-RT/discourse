use std::time::Duration;
use reqwest::blocking::Client;

pub struct StatusCheck {
    url: String,
    status: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HumanStatus {
    ConnectionError,
    Success,
    ClientError,
    ServerError,
    UnknownError,
}

impl StatusCheck {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            status: -1,
        }
    }

    pub fn is_ok(&mut self) -> bool {
        let s = self.status();
        s > 199 && s < 300
    }

    pub fn status(&mut self) -> i32 {
        if self.status == -1 {
            self.check();
        }
        self.status
    }

    pub fn human_status(&mut self) -> HumanStatus {
        let s = self.status();
        match s {
            0 => HumanStatus::ConnectionError,
            200..=299 => HumanStatus::Success,
            400..=499 => HumanStatus::ClientError,
            500..=599 => HumanStatus::ServerError,
            _ => HumanStatus::UnknownError,
        }
    }

    fn check(&mut self) {
        if let Ok(client) = Client::builder().timeout(Duration::from_secs(5)).build() {
            match client.get(&self.url).send() {
                Ok(res) => {
                    self.status = res.status().as_u16() as i32;
                }
                Err(_) => {
                    self.status = 0;
                }
            }
        } else {
            self.status = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_status() {
        let mut sc = StatusCheck::new("http://example.com");
        sc.status = 200;
        assert_eq!(sc.human_status(), HumanStatus::Success);

        sc.status = 404;
        assert_eq!(sc.human_status(), HumanStatus::ClientError);

        sc.status = 500;
        assert_eq!(sc.human_status(), HumanStatus::ServerError);

        sc.status = 0;
        assert_eq!(sc.human_status(), HumanStatus::ConnectionError);

        sc.status = 100;
        assert_eq!(sc.human_status(), HumanStatus::UnknownError);
    }

    #[test]
    fn test_is_ok() {
        let mut sc = StatusCheck::new("http://example.com");
        sc.status = 200;
        assert!(sc.is_ok());

        sc.status = 404;
        assert!(!sc.is_ok());
    }
}
