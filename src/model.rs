use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Serialize, Serializer};

/// Normalized reading state derived from Apple Books fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BookStatus {
    Finished,
    InProgress,
    NotStartedOrUnknown,
}

impl BookStatus {
    pub(crate) fn derive(
        is_finished: Option<i64>,
        finished_date_present: bool,
        progress: Option<f64>,
    ) -> Self {
        if is_finished == Some(1) || finished_date_present {
            Self::Finished
        } else if progress.unwrap_or(0.0) > 0.0 {
            Self::InProgress
        } else {
            Self::NotStartedOrUnknown
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::InProgress => "in_progress",
            Self::NotStartedOrUnknown => "not_started_or_unknown",
        }
    }
}

/// Normalized export record emitted by the CLI.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct BookRecord {
    pub(crate) title: String,
    pub(crate) author: Option<String>,
    pub(crate) status: BookStatus,
    pub(crate) reading_progress: Option<f64>,
    pub(crate) high_watermark_progress: Option<f64>,
    #[serde(serialize_with = "crate::model::serialize_optional_datetime")]
    pub(crate) finished_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::model::serialize_optional_datetime")]
    pub(crate) last_opened_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::model::serialize_optional_datetime")]
    pub(crate) last_engaged_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::model::serialize_optional_datetime")]
    pub(crate) library_record_created_at: Option<DateTime<Utc>>,
    pub(crate) asset_guid: Option<String>,
    pub(crate) genre: Option<String>,
}

pub(crate) const APPLE_EPOCH_OFFSET_SECONDS: f64 = 978_307_200.0;

pub(crate) fn apple_timestamp_to_datetime(value: Option<f64>) -> Option<DateTime<Utc>> {
    let value = value?;
    if !value.is_finite() {
        return None;
    }

    let unix_millis = ((value + APPLE_EPOCH_OFFSET_SECONDS) * 1000.0).round();
    if unix_millis < i64::MIN as f64 || unix_millis > i64::MAX as f64 {
        return None;
    }

    DateTime::from_timestamp_millis(unix_millis as i64)
}

pub(crate) fn format_datetime(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub(crate) fn serialize_optional_datetime<S>(
    value: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&format_datetime(value)),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::BookStatus;
    use super::{apple_timestamp_to_datetime, format_datetime};

    #[test]
    fn derives_finished_from_flag() {
        assert_eq!(
            BookStatus::derive(Some(1), false, Some(0.0)),
            BookStatus::Finished
        );
    }

    #[test]
    fn derives_finished_from_completion_date() {
        assert_eq!(
            BookStatus::derive(Some(0), true, Some(0.0)),
            BookStatus::Finished
        );
    }

    #[test]
    fn derives_in_progress_from_progress() {
        assert_eq!(
            BookStatus::derive(Some(0), false, Some(0.25)),
            BookStatus::InProgress
        );
    }

    #[test]
    fn derives_not_started_for_null_or_zero_progress() {
        assert_eq!(
            BookStatus::derive(Some(0), false, None),
            BookStatus::NotStartedOrUnknown
        );
        assert_eq!(
            BookStatus::derive(Some(0), false, Some(0.0)),
            BookStatus::NotStartedOrUnknown
        );
    }

    #[test]
    fn converts_apple_epoch_zero() {
        let dt = apple_timestamp_to_datetime(Some(0.0)).expect("timestamp should convert");
        assert_eq!(format_datetime(&dt), "2001-01-01T00:00:00Z");
    }

    #[test]
    fn handles_null_timestamp() {
        assert_eq!(apple_timestamp_to_datetime(None), None);
    }

    #[test]
    fn handles_invalid_float_safely() {
        assert_eq!(apple_timestamp_to_datetime(Some(f64::NAN)), None);
    }

    #[test]
    fn matches_known_expected_value() {
        let expected = chrono::Utc
            .with_ymd_and_hms(2023, 10, 11, 21, 58, 28)
            .single()
            .expect("valid date");
        let apple_seconds = expected.timestamp() - 978_307_200;

        let actual = apple_timestamp_to_datetime(Some(apple_seconds as f64))
            .expect("timestamp should convert");

        assert_eq!(actual, expected);
    }
}
