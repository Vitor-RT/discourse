use chrono::{DateTime, Datelike, Utc};

pub fn age_words(secs: Option<u64>) -> String {
    match secs {
        None => "&mdash;".to_string(),
        Some(s) => {
            let now = Utc::now();
            let to_time = now + chrono::Duration::seconds(s as i64);
            distance_of_time_in_words(&now, &to_time, false)
        }
    }
}

pub fn distance_of_time_in_words(
    from_time: &DateTime<Utc>,
    to_time: &DateTime<Utc>,
    include_seconds: bool,
) -> String {
    let distance = (to_time.timestamp() - from_time.timestamp()).abs();
    let distance_in_minutes = (distance as f64 / 60.0).round() as i64;
    let distance_in_seconds = distance;

    match distance_in_minutes {
        0..=1 => {
            if !include_seconds {
                if distance_in_minutes == 0 {
                    return "less than a minute".to_string();
                } else {
                    return "1 minute".to_string();
                }
            }

            match distance_in_seconds {
                0..=4 => "less than 5 seconds".to_string(),
                5..=9 => "less than 10 seconds".to_string(),
                10..=19 => "less than 20 seconds".to_string(),
                20..=39 => "half a minute".to_string(),
                40..=59 => "less than a minute".to_string(),
                _ => "1 minute".to_string(),
            }
        }
        2..=44 => format!("{} minutes", distance_in_minutes),
        45..=89 => "about 1 hour".to_string(),
        90..=1439 => format!("about {} hours", (distance_in_minutes as f64 / 60.0).round() as i64),
        1440..=2519 => "1 day".to_string(),
        2520..=129_599 => format!("{} days", (distance_in_minutes as f64 / 1440.0).round() as i64),
        129_600..=525_599 => format!("{} months", (distance_in_minutes as f64 / 43200.0).round() as i64),
        _ => {
            let mut fyear = from_time.year();
            if from_time.month() >= 3 {
                fyear += 1;
            }
            let mut tyear = to_time.year();
            if to_time.month() < 3 {
                tyear -= 1;
            }

            let leap_years = if fyear > tyear {
                0
            } else {
                (fyear..=tyear).filter(|&y| is_leap_year(y)).count() as i64
            };

            let minute_offset_for_leap_year = leap_years * 1440;
            let minutes_with_offset = distance_in_minutes - minute_offset_for_leap_year;
            let remainder = minutes_with_offset % 525_600;
            let distance_in_years = minutes_with_offset / 525_600;

            if remainder < 131_400 {
                format!("about {} years", distance_in_years)
            } else if remainder < 394_200 {
                format!("over {} years", distance_in_years)
            } else {
                format!("almost {} years", distance_in_years + 1)
            }
        }
    }
}

pub fn time_ago_in_words(from_time: &DateTime<Utc>, include_seconds: bool) -> String {
    distance_of_time_in_words(from_time, &Utc::now(), include_seconds)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_distance_of_time_in_words() {
        let from = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let to_1min = from + chrono::Duration::seconds(45);

        assert_eq!(distance_of_time_in_words(&from, &to_1min, false), "1 minute");
        assert_eq!(distance_of_time_in_words(&from, &to_1min, true), "less than a minute");

        let to_2h = from + chrono::Duration::hours(2);
        assert_eq!(distance_of_time_in_words(&from, &to_2h, false), "about 2 hours");

        let to_1day = from + chrono::Duration::hours(25);
        assert_eq!(distance_of_time_in_words(&from, &to_1day, false), "1 day");

        let to_5days = from + chrono::Duration::days(5);
        assert_eq!(distance_of_time_in_words(&from, &to_5days, false), "5 days");

        let to_3months = from + chrono::Duration::days(90);
        assert_eq!(distance_of_time_in_words(&from, &to_3months, false), "3 months");

        let to_2years = from + chrono::Duration::days(731);
        assert_eq!(distance_of_time_in_words(&from, &to_2years, false), "about 2 years");
    }

    #[test]
    fn test_age_words() {
        assert_eq!(age_words(None), "&mdash;");
        assert!(age_words(Some(60)).contains("minute"));
    }
}
