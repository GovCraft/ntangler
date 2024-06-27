use std::cmp::Ordering;
use std::fmt;

use crate::models::semver_impact::SemVerImpact;
use crate::models::traits::TanglerModel;
use chrono::format::StrftimeItems;
use chrono::{DateTime, TimeZone, Utc};
use console::style;
use derive_new::new;
use git2::Time;
use serde::Deserialize;
use tracing::{info, instrument};
/// A struct representing a timestamp in UTC.
#[derive(new, Clone, Default, Debug, Eq, PartialEq)]
pub(crate) struct TimeStamp(DateTime<Utc>);

impl TanglerModel for TimeStamp {}

// impl TimeStamp {
//     /// Creates a new `TimeStamp` instance with the current UTC time.
//     ///
//     /// This function captures the current time and logs the event.
//     #[instrument(level = "info")]
//     pub(crate) fn new() -> TimeStamp {
//         let now = Utc::now();
//
//         // Event: TimeStamp Created
//         // Description: Triggered when a new TimeStamp instance is created.
//         // Context: The current UTC time.
//         info!(timestamp = %now, "TimeStamp instance created");
//
//     }
// }

impl From<&Time> for TimeStamp {
    fn from(value: &Time) -> Self {
        let timestamp = value.seconds();
        let offset_minutes = value.offset_minutes();

        let datetime = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .expect("Invalid timestamp")
            + chrono::Duration::minutes(offset_minutes.into());
        TimeStamp::new(datetime)
    }
}

fn convert_to_datetime(time: &Time) -> DateTime<Utc> {
    let timestamp = time.seconds();
    let offset_minutes = time.offset_minutes();
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .expect("Invalid timestamp")
        + chrono::Duration::minutes(offset_minutes.into())
}

impl Ord for TimeStamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for TimeStamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&str> for TimeStamp {
    fn from(s: &str) -> Self {
        match DateTime::parse_from_rfc3339(s) {
            Ok(dt) => TimeStamp(dt.with_timezone(&Utc)),
            Err(_) => TimeStamp(Utc::now()), // Default to current time on parse error
        }
    }
}

impl fmt::Display for TimeStamp {
    /// Formats the `TimeStamp` for display.
    ///
    /// This method converts the stored `DateTime<Utc>` to a string in the format "YYYY-MM-DD HH:MM:SS".
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let when = self
            .0
            .format_with_items(StrftimeItems::new("%H:%M:%S"))
            .to_string();
        write!(f, "{}", when)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_new_timestamp() {
        let timestamp = TimeStamp::new();
        let now = Utc::now();

        // Verify the timestamp is created within the current Utc time.
        // Should be within a reasonable range (±5 seconds) to account for execution delay.
        let diff = (now - timestamp.0).num_seconds().abs();
        assert!(diff <= 5, "Timestamp is not within the expected range");
    }

    #[test]
    #[traced_test]
    fn test_display_timestamp() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap();
        let timestamp = TimeStamp(dt);
        let expected_output = "[10:00:00]";
        assert_eq!(timestamp.to_string(), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_default_timestamp() {
        let now = Utc::now();
        let timestamp = TimeStamp(now);

        // Use the captured time for the default timestamp.
        let diff = (Utc::now() - timestamp.0).num_seconds().abs();
        assert!(
            diff <= 5,
            "Default timestamp is not within the expected range"
        );
    }

    #[test]
    #[traced_test]
    fn test_timestamp_precision() {
        let naive_dt = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_micro_opt(10, 0, 0, 500_000)
            .unwrap();
        let dt = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
        let timestamp = TimeStamp(dt);
        let expected_output = "[10:00:00]";
        // Should round down to the nearest second.
        assert_eq!(timestamp.to_string(), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_display_timestamp_with_nanos() {
        let naive_dt = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_nano_opt(10, 0, 0, 1_000_000)
            .unwrap();
        let dt = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
        let timestamp = TimeStamp(dt);
        let expected_output = "[10:00:00]";
        // Should round down to the nearest second.
        assert_eq!(timestamp.to_string(), expected_output);
    }
}
