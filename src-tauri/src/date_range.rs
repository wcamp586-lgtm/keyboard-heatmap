//! Calendar selection over existing UTC hour buckets. No event timestamps.
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeZone};

use crate::{models::DashboardRange, storage::utc_hour_start};

#[derive(Clone, Debug)]
pub struct DateSelection {
    pub range: DashboardRange,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl From<DashboardRange> for DateSelection {
    fn from(range: DashboardRange) -> Self {
        Self {
            range,
            start_date: None,
            end_date: None,
        }
    }
}

pub struct ResolvedRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_utc: i64,
    pub end_utc: i64,
}

pub fn parse_date(value: &str) -> Result<NaiveDate> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| anyhow!("日期格式应为 YYYY-MM-DD"))?;
    if date.format("%Y-%m-%d").to_string() != value || date.year() < 1970 {
        bail!("日期必须为 1970 年以后的有效日期");
    }
    Ok(date)
}

pub fn midnight_utc<T: TimeZone>(date: NaiveDate, zone: &T) -> Result<i64> {
    let midnight = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow!("日期无效"))?;
    // Some zones skip midnight, or even a full calendar date. Advance to the
    // first valid local instant; repeated midnights include both occurrences.
    for minute in 0..=1_440 {
        match zone.from_local_datetime(&(midnight + Duration::minutes(minute))) {
            LocalResult::Single(value) => return Ok(value.timestamp()),
            LocalResult::Ambiguous(a, b) => return Ok(a.timestamp().min(b.timestamp())),
            LocalResult::None => {}
        }
    }
    bail!("无法解析本机时区中的日期")
}

impl DateSelection {
    pub fn resolve<T: TimeZone>(
        &self,
        now: &DateTime<T>,
        first_bucket: Option<i64>,
    ) -> Result<ResolvedRange> {
        let today = now.date_naive();
        let zone = now.timezone();
        let (start_date, end_date) = match self.range {
            DashboardRange::Today => (today, today),
            DashboardRange::Yesterday => (today - Duration::days(1), today - Duration::days(1)),
            DashboardRange::Week => (
                today - Duration::days(i64::from(today.weekday().num_days_from_monday())),
                today,
            ),
            DashboardRange::Month => (today.with_day(1).expect("valid month"), today),
            DashboardRange::Last30 => (today - Duration::days(29), today),
            DashboardRange::All => (
                first_bucket
                    .and_then(|stamp| zone.timestamp_opt(stamp, 0).single())
                    .map(|time| time.date_naive().min(today))
                    .unwrap_or(today),
                today,
            ),
            DashboardRange::Custom => (
                parse_date(
                    self.start_date
                        .as_deref()
                        .ok_or_else(|| anyhow!("请选择开始日期"))?,
                )?,
                parse_date(
                    self.end_date
                        .as_deref()
                        .ok_or_else(|| anyhow!("请选择结束日期"))?,
                )?,
            ),
        };
        if start_date > end_date {
            bail!("开始日期不能晚于结束日期");
        }
        if end_date > today {
            bail!("不能查询未来日期");
        }
        let next_day = end_date.succ_opt().ok_or_else(|| anyhow!("日期超出范围"))?;
        Ok(ResolvedRange {
            start_date,
            end_date,
            start_utc: midnight_utc(start_date, &zone)?.max(0),
            end_utc: midnight_utc(next_day, &zone)?.min(utc_hour_start(now.timestamp()) + 3_600),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn calendar_ranges_include_yesterday_leap_day_and_current_month() {
        let zone = FixedOffset::east_opt(8 * 3_600).unwrap();
        let now = zone.with_ymd_and_hms(2024, 3, 1, 12, 30, 0).unwrap();
        let yesterday = DateSelection::from(DashboardRange::Yesterday)
            .resolve(&now, None)
            .unwrap();
        assert_eq!(yesterday.start_date.to_string(), "2024-02-29");
        assert_eq!(yesterday.end_utc - yesterday.start_utc, 86_400);
        let month = DateSelection::from(DashboardRange::Month)
            .resolve(&now, None)
            .unwrap();
        assert_eq!(month.start_date.to_string(), "2024-03-01");
        let last30 = DateSelection::from(DashboardRange::Last30)
            .resolve(&now, None)
            .unwrap();
        assert_eq!((last30.end_date - last30.start_date).num_days(), 29);
    }

    #[test]
    fn rejects_invalid_reversed_and_future_dates() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 9, 7, 12, 0, 0).unwrap();
        for (start, end) in [
            ("2026-02-30", "2026-03-01"),
            ("2026-09-07", "2026-09-06"),
            ("2026-09-07", "2026-09-08"),
            ("2026-9-01", "2026-09-07"),
        ] {
            let selection = DateSelection {
                range: DashboardRange::Custom,
                start_date: Some(start.into()),
                end_date: Some(end.into()),
            };
            assert!(selection.resolve(&now, None).is_err());
        }
    }
}
