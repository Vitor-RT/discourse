pub fn time_left(available_in: isize) -> String {
    if available_in <= 3 {
        "short_time".to_string()
    } else if available_in < 60 {
        format!("{} seconds", available_in)
    } else if available_in < 3600 {
        format!("{} minutes", available_in / 60)
    } else {
        format!("{} hours", available_in / 3600)
    }
}

#[derive(Debug, Clone)]
pub struct LimitExceeded {
    pub available_in: isize,
    pub limit_type: Option<String>,
    pub error_code: Option<String>,
}

impl LimitExceeded {
    pub fn new(
        available_in: isize,
        limit_type: Option<String>,
        error_code: Option<String>,
    ) -> Self {
        Self {
            available_in,
            limit_type,
            error_code,
        }
    }

    pub fn time_left(&self) -> String {
        time_left(self.available_in)
    }

    pub fn description(&self) -> String {
        if let Some(ref t) = self.limit_type {
            let type_key = t.replace('-', "_");
            return format!("rate_limiter.by_type.{}", type_key);
        }
        format!("rate_limiter.too_many_requests ({})", self.time_left())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_left() {
        assert_eq!(time_left(2), "short_time");
        assert_eq!(time_left(45), "45 seconds");
        assert_eq!(time_left(120), "2 minutes");
        assert_eq!(time_left(7200), "2 hours");
    }
}
