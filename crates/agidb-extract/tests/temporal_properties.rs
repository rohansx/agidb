//! Parse natural-language time anchors into TimeRange.
//! Anchor for all tests: 2026-05-23 12:00 UTC (a Saturday).

use agidb_extract::temporal::parse_time_anchor;
use chrono::{Datelike, TimeZone, Utc};

fn anchor() -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0).unwrap()
}

#[test]
fn yesterday_returns_a_range_one_day_back() {
    let r = parse_time_anchor("yesterday", anchor()).expect("parsed");
    assert!(r.start < anchor());
    let one_day_ago = anchor() - chrono::Duration::days(1);
    let diff = (r.start - one_day_ago).num_hours().abs();
    assert!(diff <= 24, "got start={:?}", r.start);
    assert!(r.end.is_some(), "yesterday should be a range, not a point");
}

#[test]
fn last_weekend_lands_in_the_prior_saturday_sunday() {
    let r = parse_time_anchor("last weekend", anchor()).expect("parsed");
    // anchor is Sat 2026-05-23, so "last weekend" should be May 16-17.
    assert!(
        r.start.date_naive() <= chrono::NaiveDate::from_ymd_opt(2026, 5, 17).unwrap(),
        "got start={:?}",
        r.start
    );
    assert!(
        r.start.date_naive() >= chrono::NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        "got start={:?}",
        r.start
    );
    let end = r.end.expect("range");
    assert!(end > r.start);
}

#[test]
fn iso_date_parses() {
    let r = parse_time_anchor("2026-01-15", anchor()).expect("parsed");
    assert_eq!(
        r.start.date_naive(),
        chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
    );
}

#[test]
fn nonsense_returns_none() {
    assert!(parse_time_anchor("frobnicated", anchor()).is_none());
    assert!(parse_time_anchor("", anchor()).is_none());
    assert!(parse_time_anchor("   ", anchor()).is_none());
}

#[test]
fn two_months_ago_lands_in_march_2026() {
    let r = parse_time_anchor("two months ago", anchor()).expect("parsed");
    assert_eq!(r.start.month(), 3, "got month={} start={:?}", r.start.month(), r.start);
}

#[test]
fn case_insensitive() {
    assert!(parse_time_anchor("YESTERDAY", anchor()).is_some());
    assert!(parse_time_anchor("Last Weekend", anchor()).is_some());
}
