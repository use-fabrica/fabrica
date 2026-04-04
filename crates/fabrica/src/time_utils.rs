//! Standalone time formatting module using the `time` crate.
//!
//! Replaces hand-rolled time functions previously in `recent_projects.rs`.

use time::OffsetDateTime;
use time::macros::format_description;

/// Returns the current UTC timestamp as ISO 8601 with `Z` suffix.
///
/// Uses `format_description!` (not `Rfc3339`) to produce `Z` instead of `+00:00`,
/// preserving backward compatibility with existing stored timestamps.
pub fn now_iso() -> String {
    let fmt = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc().format(&fmt).unwrap_or_default()
}

/// Converts an ISO 8601 timestamp string to a human-readable relative string.
///
/// Accepts both `Z` and `+00:00` suffixes via RFC 3339 parsing.
/// Returns `"unknown"` on parse failure.
pub fn format_relative_time(last_opened: &str) -> String {
    let parsed_time =
        match OffsetDateTime::parse(last_opened, &time::format_description::well_known::Rfc3339) {
            Ok(t) => t,
            Err(_) => return "unknown".to_string(),
        };

    let elapsed = OffsetDateTime::now_utc() - parsed_time;

    if elapsed.is_negative() {
        return "just now".to_string();
    }

    let abs_diff = elapsed.whole_seconds();

    if abs_diff < 60 {
        "just now".to_string()
    } else if abs_diff < 3600 {
        format!("{}m ago", abs_diff / 60)
    } else if abs_diff < 86400 {
        format!("{}h ago", abs_diff / 3600)
    } else if abs_diff < 2592000 {
        format!("{}d ago", abs_diff / 86400)
    } else {
        "long ago".to_string()
    }
}

/// Parses an ISO 8601 / RFC 3339 timestamp string into an `OffsetDateTime`.
#[allow(dead_code)]
pub(crate) fn parse_iso(iso_str: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(iso_str, &time::format_description::well_known::Rfc3339).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_relative_time() {
        // Far in the past → "long ago"
        let result = format_relative_time("2020-01-01T00:00:00Z");
        assert_eq!(result, "long ago");

        // Timestamp from a few minutes ago (5 minutes = 300 seconds)
        let five_min_ago = (OffsetDateTime::now_utc() - time::Duration::seconds(300))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let result = format_relative_time(&five_min_ago);
        assert_eq!(result, "5m ago");
    }

    #[test]
    fn test_format_relative_time_malformed() {
        assert_eq!(format_relative_time("not-a-timestamp"), "unknown");
    }

    #[test]
    fn test_format_relative_time_empty() {
        assert_eq!(format_relative_time(""), "unknown");
    }

    #[test]
    fn test_now_iso_format() {
        let iso = now_iso();
        // Verify format: YYYY-MM-DDTHH:MM:SSZ (Z suffix, not +00:00)
        assert!(
            iso.starts_with('2'),
            "ISO string should start with century: {iso}"
        );
        assert!(
            iso.ends_with('Z'),
            "ISO string must end with Z suffix: {iso}"
        );
        // Check length: 2025-01-15T12:00:00Z = 20 chars
        assert_eq!(iso.len(), 20, "ISO string should be 20 chars: {iso}");
        // Validate all positions
        assert_eq!(&iso[4..5], "-", "position 4 should be '-': {iso}");
        assert_eq!(&iso[7..8], "-", "position 7 should be '-': {iso}");
        assert_eq!(&iso[10..11], "T", "position 10 should be 'T': {iso}");
        assert_eq!(&iso[13..14], ":", "position 13 should be ':': {iso}");
        assert_eq!(&iso[16..17], ":", "position 16 should be ':': {iso}");
    }

    #[test]
    fn test_parse_iso_valid() {
        let result = parse_iso("2025-01-15T12:00:00Z");
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month() as u8, 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 12);
    }

    #[test]
    fn test_parse_iso_with_offset() {
        let result = parse_iso("2025-01-15T12:00:00+05:30");
        assert!(result.is_some());
    }

    #[test]
    fn test_format_relative_time_future() {
        // Construct a timestamp 1 hour in the future
        let future = OffsetDateTime::now_utc() + time::Duration::hours(1);
        let future_str = future
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let result = format_relative_time(&future_str);
        assert_eq!(result, "just now");
    }
}
