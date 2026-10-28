use chrono::{NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool, sqlite::SqlitePoolOptions};
use std::time::Duration;

use crate::Result;

#[derive(Clone)]
pub(crate) struct Store {
    pool: SqlitePool,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct Subscription {
    pub room_id: String,
    pub room_name: String,
    pub meter_id: String,
    pub subscribed_at: String,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct BackfillJob {
    pub room_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub updated_at: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct BalanceRow {
    pub room_id: String,
    pub date: String,
    pub balance: f64,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct UsageRow {
    pub room_id: String,
    pub date: String,
    pub usage: f64,
    pub cost: f64,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct TrendRow {
    pub date: String,
    pub value: f64,
}

impl Store {
    pub(crate) async fn connect(database_url: &str) -> Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(10))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub(crate) async fn insert_subscription(
        &self,
        room_id: &str,
        room_name: &str,
        meter_id: &str,
        initial_balance: f64,
        balance_date: &str,
    ) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            "INSERT OR IGNORE INTO subscriptions(room_id, room_name, meter_id, subscribed_at) VALUES (?, ?, ?, ?)",
        )
        .bind(room_id).bind(room_name).bind(meter_id).bind(&now)
        .execute(&mut *tx).await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(false);
        }
        sqlx::query(
            "INSERT INTO backfill_jobs(room_id, status, updated_at) VALUES (?, 'pending', ?)",
        )
        .bind(room_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO daily_balance_records(room_id, date, balance) VALUES (?, ?, ?)")
            .bind(room_id)
            .bind(balance_date)
            .bind(initial_balance)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }

    pub(crate) async fn get_subscription(&self, room_id: &str) -> Result<Option<Subscription>> {
        Ok(sqlx::query_as::<_, Subscription>(
            "SELECT room_id, room_name, meter_id, subscribed_at FROM subscriptions WHERE room_id = ?",
        )
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub(crate) async fn subscriptions(&self) -> Result<Vec<Subscription>> {
        Ok(sqlx::query_as::<_, Subscription>(
            "SELECT room_id, room_name, meter_id, subscribed_at FROM subscriptions ORDER BY room_id",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub(crate) async fn pending_backfills(&self) -> Result<Vec<BackfillJob>> {
        sqlx::query(
            "UPDATE backfill_jobs SET status = 'pending', updated_at = ? WHERE status = 'running'",
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(sqlx::query_as::<_, BackfillJob>("SELECT room_id, status, started_at, updated_at, error FROM backfill_jobs WHERE status = 'pending'")
            .fetch_all(&self.pool).await?)
    }

    pub(crate) async fn backfill_job(&self, room_id: &str) -> Result<Option<BackfillJob>> {
        Ok(sqlx::query_as::<_, BackfillJob>("SELECT room_id, status, started_at, updated_at, error FROM backfill_jobs WHERE room_id = ?")
            .bind(room_id).fetch_optional(&self.pool).await?)
    }

    pub(crate) async fn set_backfill_status(
        &self,
        room_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE backfill_jobs SET status = ?, started_at = COALESCE(started_at, ?), updated_at = ?, error = ? WHERE room_id = ?")
            .bind(status).bind(&now).bind(&now).bind(error).bind(room_id).execute(&self.pool).await?;
        Ok(())
    }

    pub(crate) async fn upsert_usage(
        &self,
        room_id: &str,
        date: &str,
        usage: f64,
        cost: f64,
    ) -> Result<()> {
        sqlx::query("INSERT INTO daily_usage_records(room_id, date, usage, cost) VALUES (?, ?, ?, ?) ON CONFLICT(room_id, date) DO UPDATE SET usage = excluded.usage, cost = excluded.cost")
            .bind(room_id).bind(date).bind(usage).bind(cost).execute(&self.pool).await?;
        Ok(())
    }

    pub(crate) async fn upsert_balance(
        &self,
        room_id: &str,
        date: &str,
        balance: f64,
    ) -> Result<()> {
        sqlx::query("INSERT INTO daily_balance_records(room_id, date, balance) VALUES (?, ?, ?) ON CONFLICT(room_id, date) DO UPDATE SET balance = excluded.balance")
            .bind(room_id).bind(date).bind(balance).execute(&self.pool).await?;
        Ok(())
    }

    pub(crate) async fn latest_balance(&self, room_id: &str) -> Result<Option<BalanceRow>> {
        Ok(sqlx::query_as::<_, BalanceRow>("SELECT room_id, date, balance FROM daily_balance_records WHERE room_id = ? ORDER BY date DESC LIMIT 1")
            .bind(room_id).fetch_optional(&self.pool).await?)
    }

    pub(crate) async fn usage_for_date(
        &self,
        room_id: &str,
        date: NaiveDate,
    ) -> Result<Option<UsageRow>> {
        Ok(sqlx::query_as::<_, UsageRow>("SELECT room_id, date, usage, cost FROM daily_usage_records WHERE room_id = ? AND date = ?")
            .bind(room_id).bind(date.to_string()).fetch_optional(&self.pool).await?)
    }

    pub(crate) async fn trend(
        &self,
        table: &str,
        column: &str,
        room_id: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<TrendRow>> {
        let sql = format!(
            "SELECT date, {column} AS value FROM {table} WHERE room_id = ? AND date >= ? AND date <= ? ORDER BY date"
        );
        Ok(sqlx::query_as::<_, TrendRow>(&sql)
            .bind(room_id)
            .bind(start.to_string())
            .bind(end.to_string())
            .fetch_all(&self.pool)
            .await?)
    }

    pub(crate) async fn room_count(&self) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM subscriptions")
            .fetch_one(&self.pool)
            .await?)
    }
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::Store;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn migrations_and_daily_upserts_work() {
        let store = Store::connect("sqlite::memory:").await.unwrap();
        assert!(
            store
                .insert_subscription("00123", "room", "meter", 10.0, "2026-07-28")
                .await
                .unwrap()
        );
        assert!(
            !store
                .insert_subscription("00123", "room", "meter", 10.0, "2026-07-28")
                .await
                .unwrap()
        );
        store
            .upsert_usage("00123", "2026-07-28", 1.5, 2.5)
            .await
            .unwrap();
        store
            .upsert_usage("00123", "2026-07-28", 3.5, 4.5)
            .await
            .unwrap();
        let usage = store
            .usage_for_date("00123", NaiveDate::from_ymd_opt(2026, 7, 28).unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!((usage.usage, usage.cost), (3.5, 4.5));
    }
}
