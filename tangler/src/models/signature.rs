use chrono::{DateTime, TimeZone, Utc};
use crate::models::TimeStamp;
use derive_more::*;
use git2::{Signature, Time};
use serde::Deserialize;



#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TangledSignature {
    name: String,
    email: String,
    time: DateTime<Utc>,
}

impl From<Signature<'_>> for TangledSignature {
    fn from(value: Signature<'_>) -> Self {
        TangledSignature {
            name: value.name().expect("No available git signature name").to_string(),
            email: value.email().expect("No available git signature email").to_string(),
            time: convert_to_datetime(&value.when()),
        }
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