//! Apple/Core Data timestamp conversion helpers.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serializer;

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

    use super::{apple_timestamp_to_datetime, format_datetime};

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
