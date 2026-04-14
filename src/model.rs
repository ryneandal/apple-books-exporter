use chrono::{DateTime, Utc};
use serde::Serialize;

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
}

/// Normalized export record emitted by the CLI.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct BookRecord {
    pub(crate) title: String,
    pub(crate) author: Option<String>,
    pub(crate) status: BookStatus,
    pub(crate) reading_progress: Option<f64>,
    pub(crate) high_watermark_progress: Option<f64>,
    #[serde(serialize_with = "crate::timeconv::serialize_optional_datetime")]
    pub(crate) finished_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::timeconv::serialize_optional_datetime")]
    pub(crate) last_opened_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::timeconv::serialize_optional_datetime")]
    pub(crate) last_engaged_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::timeconv::serialize_optional_datetime")]
    pub(crate) library_record_created_at: Option<DateTime<Utc>>,
    pub(crate) asset_id: Option<i64>,
    pub(crate) asset_guid: Option<String>,
    pub(crate) store_id: Option<i64>,
    pub(crate) genre: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::BookStatus;

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
}
