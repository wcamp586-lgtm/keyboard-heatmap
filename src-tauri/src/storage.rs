//! SQLite persistence for privacy-preserving keyboard aggregates.
//!
//! Each of the two user-data tables has exactly three columns: an UTC hour
//! bucket, a fixed numeric identity, and an accumulated count. There are no
//! event, sequence, character, device, process, or window columns or tables.

use std::{collections::BTreeMap, path::Path, sync::Mutex};

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Duration, Local, LocalResult, TimeZone, Timelike};
use rusqlite::{params, Connection};

use crate::{
    keymap::KeyId,
    models::{DashboardRange, DashboardSnapshot, HourlyCount, KeyCount, ShortcutCount},
    shortcuts::ShortcutId,
};

pub type HourlyKeyBatch = BTreeMap<(i64, KeyId), u64>;
pub type HourlyShortcutBatch = BTreeMap<(i64, ShortcutId), u64>;

/// Bumps when a persisted shortcut identity range changes. V1 did not set a
/// user version, so its existing aggregate table reports zero and is rebuilt
/// in-place without touching its hour/count rows.
const SHORTCUT_SCHEMA_VERSION: i64 = 2;

pub struct AggregateStore {
    connection: Mutex<Connection>,
}

fn shortcut_table_sql(table_name: &str) -> String {
    // The caller supplies one of two compile-time table names, never user
    // input. Numeric shortcut IDs keep the persisted representation opaque.
    let maximum_shortcut_id = ShortcutId::COUNT - 1;
    format!(
        "
        CREATE TABLE {table_name} (
            hour_start_utc INTEGER NOT NULL
                CHECK(hour_start_utc >= 0 AND hour_start_utc % 3600 = 0),
            shortcut_id INTEGER NOT NULL
                CHECK(shortcut_id BETWEEN 0 AND {maximum_shortcut_id}),
            press_count INTEGER NOT NULL
                CHECK(press_count > 0),
            PRIMARY KEY (hour_start_utc, shortcut_id)
        ) WITHOUT ROWID;
        "
    )
}

impl AggregateStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Could not create local data directory at {}",
                    parent.display()
                )
            })?;
        }

        let mut connection = Connection::open(path).with_context(|| {
            format!(
                "Could not open local aggregate database at {}",
                path.display()
            )
        })?;
        Self::configure_and_migrate(&mut connection)?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    #[cfg(test)]
    pub(crate) fn open_in_memory() -> Result<Self> {
        let mut connection = Connection::open_in_memory()?;
        Self::configure_and_migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn configure_and_migrate(connection: &mut Connection) -> Result<()> {
        // WAL contains only pages for the fixed aggregate tables below.
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS key_hourly_counts (
                hour_start_utc INTEGER NOT NULL
                    CHECK(hour_start_utc >= 0 AND hour_start_utc % 3600 = 0),
                key_id INTEGER NOT NULL
                    CHECK(key_id BETWEEN 0 AND 103),
                press_count INTEGER NOT NULL
                    CHECK(press_count > 0),
                PRIMARY KEY (hour_start_utc, key_id)
            ) WITHOUT ROWID;

            ",
        )?;
        Self::migrate_shortcut_catalog(connection)?;
        Ok(())
    }

    /// Migrates the old 0..17 shortcut identity constraint without losing its
    /// aggregate rows. SQLite cannot alter a CHECK constraint, so the scoped
    /// replacement table is copied, swapped, and versioned in one transaction.
    fn migrate_shortcut_catalog(connection: &mut Connection) -> Result<()> {
        let shortcut_table_exists = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'shortcut_hourly_counts')",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        let schema_version =
            connection.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;

        if !shortcut_table_exists {
            connection.execute_batch(&shortcut_table_sql("shortcut_hourly_counts"))?;
        } else if schema_version < SHORTCUT_SCHEMA_VERSION {
            let transaction = connection.transaction()?;
            transaction.execute_batch(&shortcut_table_sql("shortcut_hourly_counts_replacement"))?;
            transaction.execute_batch(
                "
                INSERT INTO shortcut_hourly_counts_replacement
                    (hour_start_utc, shortcut_id, press_count)
                SELECT hour_start_utc, shortcut_id, press_count
                FROM shortcut_hourly_counts;
                DROP TABLE shortcut_hourly_counts;
                ALTER TABLE shortcut_hourly_counts_replacement
                RENAME TO shortcut_hourly_counts;
                ",
            )?;
            transaction.commit()?;
        }

        // `user_version` is schema metadata only. It contains no keyboard or
        // user data and makes the migration idempotent on future starts.
        connection.execute_batch(&format!("PRAGMA user_version = {SHORTCUT_SCHEMA_VERSION};"))?;
        Ok(())
    }

    /// Increments physical keys and fixed shortcut aggregates in one SQLite
    /// transaction. The caller provides only pre-aggregated hour/identity
    /// pairs; no raw key event can reach this boundary.
    pub fn increment_batch(
        &self,
        key_batch: &HourlyKeyBatch,
        shortcut_batch: &HourlyShortcutBatch,
    ) -> Result<()> {
        if key_batch.is_empty() && shortcut_batch.is_empty() {
            return Ok(());
        }

        let mut connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("Local aggregate database lock was poisoned"))?;
        let transaction = connection.transaction()?;
        if !key_batch.is_empty() {
            let mut statement = transaction.prepare(
                "
                INSERT INTO key_hourly_counts (hour_start_utc, key_id, press_count)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(hour_start_utc, key_id)
                DO UPDATE SET press_count = press_count + excluded.press_count
                ",
            )?;

            for (&(hour_start_utc, key_id), &press_count) in key_batch {
                let count = i64::try_from(press_count)
                    .map_err(|_| anyhow!("Press count exceeds SQLite integer capacity"))?;
                statement.execute(params![hour_start_utc, key_id.storage_id(), count])?;
            }
        }
        if !shortcut_batch.is_empty() {
            let mut statement = transaction.prepare(
                "
                INSERT INTO shortcut_hourly_counts (hour_start_utc, shortcut_id, press_count)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(hour_start_utc, shortcut_id)
                DO UPDATE SET press_count = press_count + excluded.press_count
                ",
            )?;

            for (&(hour_start_utc, shortcut_id), &press_count) in shortcut_batch {
                let count = i64::try_from(press_count)
                    .map_err(|_| anyhow!("Shortcut count exceeds SQLite integer capacity"))?;
                statement.execute(params![hour_start_utc, shortcut_id.storage_id(), count])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn dashboard(&self, range: DashboardRange) -> Result<DashboardSnapshot> {
        let (start, end) = range.bounds_now();
        self.dashboard_between(range, start, end)
    }

    pub(crate) fn dashboard_between(
        &self,
        range: DashboardRange,
        start: i64,
        end: i64,
    ) -> Result<DashboardSnapshot> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("Local aggregate database lock was poisoned"))?;
        let key_rows = {
            let mut statement = connection.prepare(
                "
                SELECT hour_start_utc, key_id, press_count
                FROM key_hourly_counts
                WHERE hour_start_utc >= ?1 AND hour_start_utc < ?2
                ORDER BY hour_start_utc, key_id
                ",
            )?;
            let rows = statement.query_map(params![start, end], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let shortcut_rows = {
            let mut statement = connection.prepare(
                "
                SELECT hour_start_utc, shortcut_id, press_count
                FROM shortcut_hourly_counts
                WHERE hour_start_utc >= ?1 AND hour_start_utc < ?2
                ORDER BY hour_start_utc, shortcut_id
                ",
            )?;
            let rows = statement.query_map(params![start, end], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let mut counts = KeyId::ALL
            .into_iter()
            .map(|key| (key, 0_u64))
            .collect::<BTreeMap<_, _>>();
        let mut hourly = [0_u64; 24];

        for (hour_start_utc, stored_key_id, stored_count) in key_rows {
            let key = KeyId::from_storage_id(stored_key_id)
                .ok_or_else(|| anyhow!("Database contains an unknown fixed key ID"))?;
            let count = u64::try_from(stored_count)
                .map_err(|_| anyhow!("Database contains an invalid aggregate count"))?;

            *counts.entry(key).or_default() += count;
            let local_hour = local_hour_from_utc(hour_start_utc);
            hourly[local_hour] += count;
        }

        let mut shortcut_counts = ShortcutId::ALL
            .into_iter()
            .map(|shortcut| (shortcut, 0_u64))
            .collect::<BTreeMap<_, _>>();
        for (_hour_start_utc, stored_shortcut_id, stored_count) in shortcut_rows {
            let shortcut = ShortcutId::from_storage_id(stored_shortcut_id)
                .ok_or_else(|| anyhow!("Database contains an unknown fixed shortcut ID"))?;
            let count = u64::try_from(stored_count)
                .map_err(|_| anyhow!("Database contains an invalid shortcut aggregate count"))?;
            *shortcut_counts.entry(shortcut).or_default() += count;
        }

        let total_presses = counts.values().sum();
        let unique_keys = counts.values().filter(|&&count| count > 0).count() as u32;
        let key_counts = KeyId::ALL
            .into_iter()
            .map(|key| KeyCount {
                key_id: key.wire_id().to_owned(),
                count: counts[&key],
            })
            .collect();
        let mut ranked = counts
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .collect::<Vec<_>>();
        ranked.sort_by(|(left_key, left_count), (right_key, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_key.storage_id().cmp(&right_key.storage_id()))
        });
        let top_keys = ranked
            .into_iter()
            .take(10)
            .map(|(key, count)| KeyCount {
                key_id: key.wire_id().to_owned(),
                count,
            })
            .collect();
        let total_shortcuts = shortcut_counts.values().sum();
        let mut ranked_shortcuts = shortcut_counts
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .collect::<Vec<_>>();
        ranked_shortcuts.sort_by(
            |(left_shortcut, left_count), (right_shortcut, right_count)| {
                right_count
                    .cmp(left_count)
                    .then_with(|| left_shortcut.storage_id().cmp(&right_shortcut.storage_id()))
            },
        );
        let top_shortcuts = ranked_shortcuts
            .into_iter()
            .take(10)
            .map(|(shortcut, count)| ShortcutCount {
                shortcut_id: shortcut.wire_id().to_owned(),
                count,
            })
            .collect();
        let hourly_distribution = hourly
            .into_iter()
            .enumerate()
            .map(|(hour, count)| HourlyCount {
                hour: hour as u8,
                count,
            })
            .collect();

        Ok(DashboardSnapshot {
            range,
            range_start_utc: start,
            range_end_utc: end,
            total_presses,
            unique_keys,
            total_shortcuts,
            key_counts,
            top_keys,
            top_shortcuts,
            hourly_distribution,
        })
    }
}

impl DashboardRange {
    fn bounds_now(self) -> (i64, i64) {
        let now = Local::now();
        let today = now.date_naive();
        let start_day = match self {
            Self::Today => today,
            Self::Week => today - Duration::days(i64::from(now.weekday().num_days_from_monday())),
            Self::Month => today.with_day(1).expect("Every month has day one"),
            // The application only writes positive Unix timestamps. Keeping the
            // lower bound at zero avoids making a special unbounded SQL query.
            Self::All => return (0, utc_hour_start(now.timestamp()) + 3_600),
        };
        let start_naive = start_day
            .and_hms_opt(0, 0, 0)
            .expect("Midnight is always a valid wall-clock time");
        let start_local = match Local.from_local_datetime(&start_naive) {
            LocalResult::Single(value) => value,
            // A repeated midnight is assigned to the earlier occurrence so a
            // local dashboard never drops the first hour on a DST transition.
            LocalResult::Ambiguous(earlier, _) => earlier,
            // This is extraordinarily rare; it remains a safe, local fallback.
            LocalResult::None => now,
        };

        (
            start_local.timestamp(),
            utc_hour_start(now.timestamp()) + 3_600,
        )
    }
}

pub fn utc_hour_start(timestamp_seconds: i64) -> i64 {
    timestamp_seconds.div_euclid(3_600) * 3_600
}

fn local_hour_from_utc(timestamp_seconds: i64) -> usize {
    match Local.timestamp_opt(timestamp_seconds, 0) {
        LocalResult::Single(value) => value.hour() as usize,
        LocalResult::Ambiguous(earlier, _) => earlier.hour() as usize,
        LocalResult::None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{utc_hour_start, AggregateStore, HourlyKeyBatch, HourlyShortcutBatch};
    use crate::{keymap::KeyId, models::DashboardRange, shortcuts::ShortcutId};

    #[test]
    fn aggregates_cross_hour_and_cross_day_without_raw_events() {
        let store = AggregateStore::open_in_memory().unwrap();
        let mut batch = HourlyKeyBatch::new();
        batch.insert((utc_hour_start(86_399), KeyId::KeyA), 2);
        batch.insert((utc_hour_start(86_400), KeyId::KeyA), 3);
        batch.insert((utc_hour_start(86_400), KeyId::Space), 1);
        let mut shortcuts = HourlyShortcutBatch::new();
        shortcuts.insert((utc_hour_start(86_399), ShortcutId::CtrlC), 1);
        shortcuts.insert((utc_hour_start(86_400), ShortcutId::CtrlC), 2);
        store.increment_batch(&batch, &shortcuts).unwrap();

        let dashboard = store
            .dashboard_between(DashboardRange::All, 0, 90_000)
            .unwrap();
        assert_eq!(dashboard.total_presses, 6);
        assert_eq!(dashboard.unique_keys, 2);
        assert_eq!(dashboard.total_shortcuts, 3);
        assert_eq!(dashboard.top_shortcuts[0].shortcut_id, "ctrl_c");
        assert_eq!(dashboard.top_shortcuts[0].count, 3);
        assert_eq!(
            dashboard
                .key_counts
                .iter()
                .find(|count| count.key_id == "key_a")
                .unwrap()
                .count,
            5
        );
    }

    #[test]
    fn key_and_shortcut_aggregates_survive_reopening_the_local_database() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("keyboard-heatmap.sqlite3");
        let mut keys = HourlyKeyBatch::new();
        keys.insert((3_600, KeyId::KeyW), 7);
        let mut shortcuts = HourlyShortcutBatch::new();
        shortcuts.insert((3_600, ShortcutId::CtrlV), 2);
        shortcuts.insert((7_200, ShortcutId::CtrlV), 5);

        AggregateStore::open(&database_path)
            .unwrap()
            .increment_batch(&keys, &shortcuts)
            .unwrap();

        let reopened = AggregateStore::open(&database_path).unwrap();
        let dashboard = reopened
            .dashboard_between(DashboardRange::All, 0, 10_800)
            .unwrap();
        assert_eq!(dashboard.total_presses, 7);
        assert_eq!(dashboard.top_keys[0].key_id, "key_w");
        assert_eq!(dashboard.total_shortcuts, 7);
        assert_eq!(dashboard.top_shortcuts[0].shortcut_id, "ctrl_v");
        assert_eq!(dashboard.top_shortcuts[0].count, 7);

        // Remove only the test directory. This never touches the user's data.
        drop(reopened);
        drop(directory);
        assert!(!database_path.exists());
    }

    #[test]
    fn opens_legacy_key_database_without_losing_key_aggregates() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("keyboard-heatmap.sqlite3");
        {
            let legacy = rusqlite::Connection::open(&database_path).unwrap();
            legacy
                .execute_batch(
                    "
                    CREATE TABLE key_hourly_counts (
                        hour_start_utc INTEGER NOT NULL,
                        key_id INTEGER NOT NULL,
                        press_count INTEGER NOT NULL,
                        PRIMARY KEY (hour_start_utc, key_id)
                    ) WITHOUT ROWID;
                    INSERT INTO key_hourly_counts (hour_start_utc, key_id, press_count)
                    VALUES (3600, 39, 7);
                    ",
                )
                .unwrap();
        }

        let migrated = AggregateStore::open(&database_path).unwrap();
        let dashboard = migrated
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_presses, 7);
        assert_eq!(dashboard.top_keys[0].key_id, "key_w");
        assert_eq!(dashboard.total_shortcuts, 0);

        let connection = migrated.connection.lock().unwrap();
        let created_shortcut_table = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'shortcut_hourly_counts'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(created_shortcut_table, 1);
    }

    #[test]
    fn migrates_v1_shortcut_constraint_and_preserves_existing_aggregates() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("keyboard-heatmap.sqlite3");
        {
            let legacy = rusqlite::Connection::open(&database_path).unwrap();
            legacy
                .execute_batch(
                    "
                    CREATE TABLE shortcut_hourly_counts (
                        hour_start_utc INTEGER NOT NULL
                            CHECK(hour_start_utc >= 0 AND hour_start_utc % 3600 = 0),
                        shortcut_id INTEGER NOT NULL
                            CHECK(shortcut_id BETWEEN 0 AND 17),
                        press_count INTEGER NOT NULL
                            CHECK(press_count > 0),
                        PRIMARY KEY (hour_start_utc, shortcut_id)
                    ) WITHOUT ROWID;
                    INSERT INTO shortcut_hourly_counts
                        (hour_start_utc, shortcut_id, press_count)
                    VALUES (3600, 1, 4);
                    ",
                )
                .unwrap();
        }

        let migrated = AggregateStore::open(&database_path).unwrap();
        let before_new_catalog_write = migrated
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(before_new_catalog_write.total_shortcuts, 4);
        assert_eq!(
            before_new_catalog_write.top_shortcuts[0].shortcut_id,
            "ctrl_c"
        );
        assert_eq!(before_new_catalog_write.top_shortcuts[0].count, 4);

        let keys = HourlyKeyBatch::new();
        let mut shortcuts = HourlyShortcutBatch::new();
        shortcuts.insert((3_600, ShortcutId::WinD), 3);
        migrated.increment_batch(&keys, &shortcuts).unwrap();

        let dashboard = migrated
            .dashboard_between(DashboardRange::All, 0, 7_200)
            .unwrap();
        assert_eq!(dashboard.total_shortcuts, 7);
        assert_eq!(dashboard.top_shortcuts[0].shortcut_id, "ctrl_c");
        assert_eq!(dashboard.top_shortcuts[1].shortcut_id, "win_d");
        assert_eq!(dashboard.top_shortcuts[1].count, 3);

        let connection = migrated.connection.lock().unwrap();
        let table_sql = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'shortcut_hourly_counts'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap();
        assert!(table_sql.contains("CHECK(shortcut_id BETWEEN 0 AND 84)"));
        let schema_version = connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap();
        assert_eq!(schema_version, super::SHORTCUT_SCHEMA_VERSION);
    }

    #[test]
    fn aggregate_tables_have_only_fixed_non_raw_input_columns() {
        let store = AggregateStore::open_in_memory().unwrap();
        let connection = store.connection.lock().unwrap();
        let mut table_statement = connection
            .prepare(
                "
                SELECT name
                FROM sqlite_master
                WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
                ORDER BY name
                ",
            )
            .unwrap();
        let table_names = table_statement
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(table_names, ["key_hourly_counts", "shortcut_hourly_counts"]);

        for (table_name, expected_columns) in [
            (
                "key_hourly_counts",
                ["hour_start_utc", "key_id", "press_count"].as_slice(),
            ),
            (
                "shortcut_hourly_counts",
                ["hour_start_utc", "shortcut_id", "press_count"].as_slice(),
            ),
        ] {
            let mut column_statement = connection
                .prepare(&format!("PRAGMA table_info({table_name})"))
                .unwrap();
            let names = column_statement
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            assert_eq!(names, expected_columns);
            for forbidden in [
                "character",
                "text",
                "event",
                "sequence",
                "device",
                "window",
                "process",
                "clipboard",
                "raw",
            ] {
                assert!(!names.iter().any(|name| name.contains(forbidden)));
            }
        }
    }
}
