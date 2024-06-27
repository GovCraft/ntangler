use std::cmp::Ordering;
use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use chrono::format::StrftimeItems;
use derive_new::new;
use git2::Time;
use tracing::instrument;

/// A struct representing a timestamp in UTC.
#[derive(new, Clone, Default, Debug, Eq, PartialEq)]
pub(crate) struct TimeStamp(DateTime<Utc>);


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

