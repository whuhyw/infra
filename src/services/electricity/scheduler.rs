use chrono::{Duration as ChronoDuration, NaiveTime, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use futures::{StreamExt, stream};
use tokio::time;

use super::{api::AppState, models::parse_api_date, store::Subscription};

pub(crate) async fn run(state: AppState) {
    loop {
        let wait =
            duration_until_next_run(state.config.scheduled_time.0, state.config.scheduled_time.1);
        tracing::info!(
            wait_seconds = wait.as_secs(),
            "next scheduled update is pending"
        );
        time::sleep(wait).await;
        let run_started = Utc::now().to_rfc3339();
        *state.last_scheduled_run.write().await = Some(run_started);
        match state.store.subscriptions().await {
            Ok(subscriptions) => {
                let concurrency = state.config.max_concurrent.max(1);
                tracing::info!(
                    rooms = subscriptions.len(),
                    concurrency,
                    "scheduled update started"
                );
                stream::iter(subscriptions)
                    .for_each_concurrent(concurrency, |subscription| {
                        update_until_success(state.clone(), subscription)
                    })
                    .await;
                tracing::info!("scheduled update completed");
            }
            Err(error) => {
                tracing::error!(%error, "failed to list subscriptions for scheduled update")
            }
        }
    }
}

fn duration_until_next_run(hour: u32, minute: u32) -> std::time::Duration {
    let now = Utc::now();
    let local = now.with_timezone(&Shanghai);
    let run_time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap_or(NaiveTime::MIN);
    let mut next_date = local.date_naive();
    let candidate = next_date.and_time(run_time);
    if candidate <= local.naive_local() {
        next_date += ChronoDuration::days(1);
    }
    let next = Shanghai
        .from_local_datetime(&next_date.and_time(run_time))
        .single()
        .expect("Asia/Shanghai local time is unambiguous")
        .with_timezone(&Utc);
    (next - now).to_std().unwrap_or_default()
}

async fn update_until_success(state: AppState, subscription: Subscription) {
    let yesterday = super::api::scheduled_today()
        .pred_opt()
        .unwrap_or_else(super::api::scheduled_today);
    loop {
        let result = async {
            let usage = state
                .whu
                .get_meter_day_value(&subscription.meter_id, yesterday, yesterday)
                .await?;
            for item in usage {
                let date = parse_api_date(&item.date).ok_or_else(|| {
                    crate::Error::InvalidData("WHU returned an invalid usage date".into())
                })?;
                state
                    .store
                    .upsert_usage(
                        &subscription.room_id,
                        &date.to_string(),
                        item.usage,
                        item.cost,
                    )
                    .await?;
            }
            let reserve = state.whu.get_reserve(&subscription.meter_id).await?;
            state
                .store
                .upsert_balance(
                    &subscription.room_id,
                    &super::api::scheduled_today().to_string(),
                    reserve.balance,
                )
                .await?;
            crate::Result::Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                tracing::info!(room_id = %subscription.room_id, "scheduled room update succeeded");
                return;
            }
            Err(error) => {
                tracing::error!(room_id = %subscription.room_id, %error, "scheduled room update failed; retrying");
                time::sleep(state.config.retry_interval).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::duration_until_next_run;

    #[test]
    fn next_run_is_within_one_day() {
        let duration = duration_until_next_run(2, 0);
        assert!(!duration.is_zero());
        assert!(duration.as_secs() <= 24 * 60 * 60);
    }
}
