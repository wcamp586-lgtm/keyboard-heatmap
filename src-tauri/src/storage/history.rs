//! Bounded calendar pages derived from existing aggregate tables only.
use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Duration, Local, Offset, TimeZone};
use rusqlite::{params, Connection};

use super::AggregateStore;
use crate::{
    date_range::{midnight_utc, DateSelection},
    keymap::KeyId,
    models::{
        DailyRecord, DashboardRange, DayDetail, HistoryPage, HistoryTarget, HourRecord, KeyCount,
    },
    shortcuts::ShortcutId,
};

const PAGE_DAYS: i64 = 30;

pub(super) fn first_bucket(connection: &Connection) -> Result<Option<i64>> {
    Ok(connection.query_row(
        "SELECT MIN(stamp) FROM (
            SELECT MIN(hour_start_utc) AS stamp FROM key_hourly_counts
            UNION ALL SELECT MIN(hour_start_utc) FROM shortcut_hourly_counts
        )",
        [],
        |row| row.get(0),
    )?)
}

#[derive(Default)]
struct Bucket {
    keys: BTreeMap<KeyId, u64>,
    shortcuts: u64,
}

impl Bucket {
    fn total(&self) -> u64 {
        self.keys.values().sum()
    }

    fn top_key(&self) -> Option<KeyCount> {
        self.keys
            .iter()
            .min_by(|(a, ac), (b, bc)| bc.cmp(ac).then_with(|| a.cmp(b)))
            .map(|(key, count)| KeyCount {
                key_id: key.wire_id().into(),
                count: *count,
            })
    }
}

/// Only fixed catalog IDs can become SQL parameters. No arbitrary chord input.
fn target_ids(target: Option<&HistoryTarget>) -> Result<(Option<i64>, Option<i64>)> {
    match target {
        None => Ok((None, None)),
        Some(HistoryTarget::Key { id }) => KeyId::ALL
            .into_iter()
            .find(|key| key.wire_id() == id)
            .map(|key| (Some(key.storage_id()), None))
            .ok_or_else(|| anyhow!("无效的固定按键")),
        Some(HistoryTarget::Shortcut { id }) => ShortcutId::ALL
            .into_iter()
            .find(|key| key.wire_id() == id)
            .map(|key| (None, Some(key.storage_id())))
            .ok_or_else(|| anyhow!("无效的固定快捷键")),
    }
}

fn read_buckets(
    connection: &Connection,
    start: i64,
    end: i64,
    target: Option<&HistoryTarget>,
) -> Result<BTreeMap<i64, Bucket>> {
    let (key_id, shortcut_id) = target_ids(target)?;
    let mut buckets = BTreeMap::<i64, Bucket>::new();
    if shortcut_id.is_none() {
        let mut statement = connection.prepare(
            "SELECT hour_start_utc, key_id, press_count FROM key_hourly_counts
             WHERE hour_start_utc >= ?1 AND hour_start_utc < ?2 AND (?3 IS NULL OR key_id = ?3)
             ORDER BY hour_start_utc, key_id",
        )?;
        let rows = statement.query_map(params![start, end, key_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, u64>(2)?,
            ))
        })?;
        for row in rows {
            let (hour, id, count) = row?;
            let key = KeyId::from_storage_id(id).ok_or_else(|| anyhow!("Unknown aggregate key"))?;
            buckets.entry(hour).or_default().keys.insert(key, count);
        }
    }
    if key_id.is_none() {
        let mut statement = connection.prepare(
            "SELECT hour_start_utc, SUM(press_count) FROM shortcut_hourly_counts
             WHERE hour_start_utc >= ?1 AND hour_start_utc < ?2 AND (?3 IS NULL OR shortcut_id = ?3)
             GROUP BY hour_start_utc ORDER BY hour_start_utc",
        )?;
        let rows = statement.query_map(params![start, end, shortcut_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, u64>(1)?))
        })?;
        for row in rows {
            let (hour, count) = row?;
            buckets.entry(hour).or_default().shortcuts = count;
        }
    }
    Ok(buckets)
}

impl AggregateStore {
    pub fn history(
        &self,
        selection: &DateSelection,
        target: Option<&HistoryTarget>,
        page: u32,
    ) -> Result<HistoryPage> {
        self.history_at(selection, target, page, &Local::now())
    }

    fn history_at<T: TimeZone>(
        &self,
        selection: &DateSelection,
        target: Option<&HistoryTarget>,
        page: u32,
        now: &DateTime<T>,
    ) -> Result<HistoryPage> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("Database lock unavailable"))?;
        let first = if selection.range == DashboardRange::All {
            first_bucket(&connection)?
        } else {
            None
        };
        let range = selection.resolve(now, first)?;
        let total_pages = ((range.end_date - range.start_date).num_days() / PAGE_DAYS + 1) as u32;
        let page = page.min(total_pages - 1);
        let page_end = range.end_date - Duration::days(i64::from(page) * PAGE_DAYS);
        let page_start = (page_end - Duration::days(PAGE_DAYS - 1)).max(range.start_date);
        let zone = now.timezone();
        let start = midnight_utc(page_start, &zone)?.max(range.start_utc);
        let end = midnight_utc(
            page_end
                .succ_opt()
                .ok_or_else(|| anyhow!("Date overflow"))?,
            &zone,
        )?
        .min(range.end_utc);
        let buckets = read_buckets(&connection, start, end, target)?;
        let mut daily = BTreeMap::new();
        let mut peaks = BTreeMap::<_, (u64, String)>::new();
        for (stamp, bucket) in buckets {
            let local = zone
                .timestamp_opt(stamp, 0)
                .single()
                .ok_or_else(|| anyhow!("Invalid hour bucket"))?;
            let date = local.date_naive();
            let total = bucket.total();
            let peak = peaks.entry(date).or_default();
            if total > peak.0 {
                *peak = (
                    total,
                    format!(
                        "{} (UTC{})",
                        local.naive_local().format("%H:%M"),
                        local.offset().fix()
                    ),
                );
            }
            let day = daily.entry(date).or_insert_with(Bucket::default);
            for (key, count) in bucket.keys {
                *day.keys.entry(key).or_default() += count;
            }
            day.shortcuts += bucket.shortcuts;
        }
        let days = (0..=(page_end - page_start).num_days())
            .map(|index| {
                let date = page_end - Duration::days(index);
                let bucket = daily.remove(&date).unwrap_or_default();
                DailyRecord {
                    date: date.to_string(),
                    total_presses: bucket.total(),
                    total_shortcuts: bucket.shortcuts,
                    unique_keys: bucket.keys.len() as u32,
                    top_key: bucket.top_key(),
                    peak_hour: peaks
                        .remove(&date)
                        .filter(|(count, _)| *count > 0)
                        .map(|(_, label)| label),
                }
            })
            .collect();
        Ok(HistoryPage {
            generated_at: now.timestamp(),
            range_start_date: range.start_date.to_string(),
            range_end_date: range.end_date.to_string(),
            page,
            total_pages,
            days,
        })
    }

    pub fn day_detail(&self, date: &str, target: Option<&HistoryTarget>) -> Result<DayDetail> {
        self.day_detail_at(date, target, &Local::now())
    }

    fn day_detail_at<T: TimeZone>(
        &self,
        date: &str,
        target: Option<&HistoryTarget>,
        now: &DateTime<T>,
    ) -> Result<DayDetail> {
        let selection = DateSelection {
            range: DashboardRange::Custom,
            start_date: Some(date.into()),
            end_date: Some(date.into()),
        };
        let range = selection.resolve(now, None)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("Database lock unavailable"))?;
        let mut buckets = read_buckets(&connection, range.start_utc, range.end_utc, target)?;
        let zone = now.timezone();
        let mut hours = Vec::new();
        // Stored UTC buckets belong to the local date of their START. This
        // remains deterministic in half-hour zones and repeated DST hours.
        let first = (range.start_utc + 3_599).div_euclid(3_600) * 3_600;
        for stamp in (first..range.end_utc).step_by(3_600) {
            let local = zone
                .timestamp_opt(stamp, 0)
                .single()
                .ok_or_else(|| anyhow!("Invalid hour bucket"))?;
            if local.date_naive() != range.start_date {
                bail!("Hour outside selected local date");
            }
            let bucket = buckets.remove(&stamp).unwrap_or_default();
            hours.push(HourRecord {
                hour_start_utc: stamp,
                label: local.naive_local().format("%H:%M").to_string(),
                utc_offset: local.offset().fix().to_string(),
                total_presses: bucket.total(),
                total_shortcuts: bucket.shortcuts,
                top_key: bucket.top_key(),
            });
        }
        Ok(DayDetail {
            generated_at: now.timestamp(),
            date: date.into(),
            hours,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{HourlyKeyBatch, HourlyShortcutBatch};
    use chrono::{FixedOffset, Utc};

    #[test]
    fn dst_days_keep_23_or_25_distinct_hours_and_matching_daily_totals() {
        let zone = chrono_tz::America::New_York;
        let store = AggregateStore::open_in_memory().unwrap();
        let now = zone.with_ymd_and_hms(2024, 12, 1, 12, 0, 0).unwrap();
        for (date, expected) in [("2024-03-10", 23), ("2024-11-03", 25)] {
            let selection = DateSelection {
                range: DashboardRange::Custom,
                start_date: Some(date.into()),
                end_date: Some(date.into()),
            };
            let range = selection.resolve(&now, None).unwrap();
            let keys = (range.start_utc..range.end_utc)
                .step_by(3600)
                .map(|stamp| ((stamp, KeyId::Space), 1))
                .collect();
            store
                .increment_batch(&keys, &HourlyShortcutBatch::new())
                .unwrap();
            let detail = store.day_detail_at(date, None, &now).unwrap();
            assert_eq!(detail.hours.len(), expected);
            assert_eq!(
                detail
                    .hours
                    .iter()
                    .map(|hour| hour.total_presses)
                    .sum::<u64>(),
                expected as u64
            );
            let page = store.history_at(&selection, None, 0, &now).unwrap();
            assert_eq!(page.days[0].total_presses, expected as u64);
            if expected == 25 {
                let repeated = detail
                    .hours
                    .iter()
                    .filter(|hour| hour.label == "01:00")
                    .collect::<Vec<_>>();
                assert_eq!(repeated.len(), 2);
                assert_ne!(repeated[0].utc_offset, repeated[1].utc_offset);
                assert_ne!(repeated[0].hour_start_utc, repeated[1].hour_start_utc);
            } else {
                assert!(!detail.hours.iter().any(|hour| hour.label == "02:00"));
            }
        }
    }

    #[test]
    fn daily_pages_include_gaps_and_targeted_hour_totals_match() {
        let store = AggregateStore::open_in_memory().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 9, 7, 12, 30, 0).unwrap();
        let stamp = Utc
            .with_ymd_and_hms(2026, 9, 6, 9, 0, 0)
            .unwrap()
            .timestamp();
        let keys = HourlyKeyBatch::from([
            ((stamp, KeyId::KeyA), 8),
            ((stamp + 3600, KeyId::KeyA), 2),
            ((stamp, KeyId::Space), 3),
        ]);
        let shortcuts = HourlyShortcutBatch::from([((stamp, ShortcutId::CtrlC), 4)]);
        store.increment_batch(&keys, &shortcuts).unwrap();
        let selection = DateSelection {
            range: DashboardRange::Custom,
            start_date: Some("2026-08-01".into()),
            end_date: Some("2026-09-07".into()),
        };
        let page = store.history_at(&selection, None, 0, &now).unwrap();
        assert_eq!(page.days.len(), 30);
        assert_eq!(page.total_pages, 2);
        assert_eq!(page.days[0].total_presses, 0);
        assert!(page.days[0].top_key.is_none());
        assert_eq!(page.days[1].total_presses, 13);
        assert_eq!(page.days[1].total_shortcuts, 4);
        assert_eq!(page.days[1].unique_keys, 2);
        assert_eq!(page.days[1].top_key.as_ref().unwrap().count, 10);
        assert!(page.days[1]
            .peak_hour
            .as_ref()
            .unwrap()
            .starts_with("09:00"));
        let last = store.history_at(&selection, None, u32::MAX, &now).unwrap();
        assert_eq!(last.page, 1);
        assert_eq!(last.days.len(), 8);
        let target = HistoryTarget::Key { id: "key_a".into() };
        let detail = store
            .day_detail_at("2026-09-06", Some(&target), &now)
            .unwrap();
        assert_eq!(detail.hours.len(), 24);
        assert_eq!(
            detail
                .hours
                .iter()
                .map(|hour| hour.total_presses)
                .sum::<u64>(),
            10
        );
        assert_eq!(
            detail
                .hours
                .iter()
                .map(|hour| hour.total_shortcuts)
                .sum::<u64>(),
            0
        );
        let shortcut = HistoryTarget::Shortcut {
            id: "ctrl_c".into(),
        };
        let detail = store
            .day_detail_at("2026-09-06", Some(&shortcut), &now)
            .unwrap();
        assert_eq!(
            detail
                .hours
                .iter()
                .map(|hour| hour.total_shortcuts)
                .sum::<u64>(),
            4
        );
        assert_eq!(
            store
                .day_detail_at("2026-09-07", None, &now)
                .unwrap()
                .hours
                .len(),
            13
        );
        assert!(store
            .day_detail_at(
                "2026-09-06",
                Some(&HistoryTarget::Key {
                    id: "typed_text".into()
                }),
                &now
            )
            .is_err());
    }

    #[test]
    fn all_range_and_half_hour_timezone_use_bucket_start_date() {
        let store = AggregateStore::open_in_memory().unwrap();
        let zone = FixedOffset::east_opt(5 * 3600 + 1800).unwrap();
        let now = zone.with_ymd_and_hms(2026, 9, 7, 12, 45, 0).unwrap();
        let stamp = Utc
            .with_ymd_and_hms(2026, 9, 6, 18, 0, 0)
            .unwrap()
            .timestamp();
        store
            .increment_batch(
                &HourlyKeyBatch::from([
                    ((stamp, KeyId::Space), 2),
                    ((stamp + 3600, KeyId::Space), 3),
                ]),
                &HourlyShortcutBatch::new(),
            )
            .unwrap();
        let page = store
            .history_at(&DashboardRange::All.into(), None, 0, &now)
            .unwrap();
        assert_eq!(page.range_start_date, "2026-09-06");
        assert_eq!(page.days[0].total_presses, 3);
        assert_eq!(page.days[1].total_presses, 2);
        let detail = store.day_detail_at("2026-09-07", None, &now).unwrap();
        assert_eq!(detail.hours[0].label, "00:30");
        assert_eq!(detail.hours[0].total_presses, 3);
        assert_eq!(detail.hours.last().unwrap().label, "12:30");
    }
}
