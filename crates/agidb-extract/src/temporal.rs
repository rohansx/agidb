//! Parse natural-language time anchors into [`TimeRange`] values.
//!
//! Explicitly handles the high-frequency relative expressions
//! ("yesterday", "last weekend", "this weekend") so we control the
//! returned ranges; everything else falls through to
//! [`chrono_english::parse_date_string`] for absolute / ISO dates and
//! "N units ago / from now" expressions.
//!
//! Returns `None` for unparseable input — the orchestrator falls back
//! to the observation time, per the phase-3 design spec § 9 (time
//! parse failures are non-fatal).

use agidb_core::types::TimeRange;
use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use chrono_english::{parse_date_string, Dialect};

/// Parse `text` as a time expression, anchored at `now`.
pub fn parse_time_anchor(text: &str, now: DateTime<Utc>) -> Option<TimeRange> {
    let text = text.trim().to_lowercase();
    if text.is_empty() {
        return None;
    }
    let text = normalize_number_words(&text);

    if text == "yesterday" {
        let start_day = (now - Duration::days(1)).date_naive();
        let start = start_day.and_hms_opt(0, 0, 0)?;
        let end = start_day.and_hms_opt(23, 59, 59)?;
        return Some(TimeRange {
            start: DateTime::<Utc>::from_naive_utc_and_offset(start, Utc),
            end: Some(DateTime::<Utc>::from_naive_utc_and_offset(end, Utc)),
        });
    }
    if text == "last weekend" {
        return last_weekend(now);
    }
    if text == "this weekend" {
        return this_weekend(now);
    }

    parse_date_string(&text, now, Dialect::Us)
        .ok()
        .map(|dt| TimeRange {
            start: dt.with_timezone(&Utc),
            end: None,
        })
}

/// The most recent Saturday strictly before `now`, paired with its Sunday.
fn last_weekend(now: DateTime<Utc>) -> Option<TimeRange> {
    let today = now.date_naive();
    let mut d = today - Duration::days(1);
    while d.weekday() != Weekday::Sat {
        d -= Duration::days(1);
    }
    let sat = d;
    let sun = sat + Duration::days(1);
    Some(TimeRange {
        start: DateTime::<Utc>::from_naive_utc_and_offset(sat.and_hms_opt(0, 0, 0)?, Utc),
        end: Some(DateTime::<Utc>::from_naive_utc_and_offset(
            sun.and_hms_opt(23, 59, 59)?,
            Utc,
        )),
    })
}

/// Map the first ten English cardinal number-words to digits so
/// `chrono_english` parses "two months ago" the same as "2 months ago".
/// NER routinely produces word-form numerals; the cost of this
/// preprocessor is tiny vs the recall it unlocks.
fn normalize_number_words(s: &str) -> String {
    let pairs = [
        ("one ", "1 "),
        ("two ", "2 "),
        ("three ", "3 "),
        ("four ", "4 "),
        ("five ", "5 "),
        ("six ", "6 "),
        ("seven ", "7 "),
        ("eight ", "8 "),
        ("nine ", "9 "),
        ("ten ", "10 "),
    ];
    let mut out = s.to_string();
    for (k, v) in pairs {
        out = out.replace(k, v);
    }
    out
}

/// The upcoming Saturday (or today if today is Sat), paired with Sunday.
fn this_weekend(now: DateTime<Utc>) -> Option<TimeRange> {
    let today = now.date_naive();
    let mut d = today;
    while d.weekday() != Weekday::Sat {
        d += Duration::days(1);
    }
    let sat = d;
    let sun = sat + Duration::days(1);
    Some(TimeRange {
        start: DateTime::<Utc>::from_naive_utc_and_offset(sat.and_hms_opt(0, 0, 0)?, Utc),
        end: Some(DateTime::<Utc>::from_naive_utc_and_offset(
            sun.and_hms_opt(23, 59, 59)?,
            Utc,
        )),
    })
}
