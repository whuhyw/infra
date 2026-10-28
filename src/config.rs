use std::{env, str::FromStr, time::Duration};

use crate::WhuApiOptions;

#[derive(Clone)]
pub(crate) struct AppConfig {
    pub bind_address: String,
    pub database_url: String,
    pub whu: WhuApiOptions,
    pub query_interval: Duration,
    pub refresh_interval: Duration,
    pub refresh_enabled: bool,
    pub scheduled_time: (u32, u32),
    pub retry_interval: Duration,
    pub max_concurrent: usize,
    pub backfill_chunk_days: i64,
}

impl AppConfig {
    pub(crate) fn from_env() -> Self {
        let defaults = WhuApiOptions::default();
        Self {
            bind_address: value("BIND_ADDRESS", "0.0.0.0:5051"),
            database_url: value("DATABASE_URL", "sqlite://electricity.db"),
            whu: WhuApiOptions {
                base_url: value("WhuApi__BaseUrl", &defaults.base_url),
                account_id: value("WhuApi__AccountId", &defaults.account_id),
                account_pass: value("WhuApi__AccountPass", &defaults.account_pass),
                sys_id: value("WhuApi__SysId", &defaults.sys_id),
                sm2_public_key: value("WhuApi__Sm2PublicKey", &defaults.sm2_public_key),
            },
            query_interval: duration("RateLimiting__QueryInterval", Duration::from_secs(3600)),
            refresh_interval: duration("RateLimiting__RefreshInterval", Duration::from_secs(1800)),
            refresh_enabled: value("RateLimiting__RefreshEnabled", "true") == "true",
            scheduled_time: parse_time(&value("ScheduledTask__RunTime", "02:00")),
            retry_interval: duration("ScheduledTask__RetryInterval", Duration::from_secs(900)),
            max_concurrent: value("ScheduledTask__MaxConcurrent", "5")
                .parse()
                .unwrap_or(5),
            backfill_chunk_days: value("Backfill__ChunkSizeDays", "100")
                .parse()
                .unwrap_or(100),
        }
    }
}

fn value(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn duration(name: &str, default: Duration) -> Duration {
    let Some(value) = env::var(name).ok() else {
        return default;
    };
    let parts: Vec<_> = value.split(':').collect();
    let seconds = (|| match parts.as_slice() {
        [hours, minutes, seconds] => Some(
            hours
                .parse::<u64>()
                .ok()?
                .saturating_mul(3600)
                .saturating_add(minutes.parse::<u64>().ok()?.saturating_mul(60))
                .saturating_add(seconds.parse::<u64>().ok()?),
        ),
        _ => None,
    })();
    seconds.map(Duration::from_secs).unwrap_or(default)
}

fn parse_time(value: &str) -> (u32, u32) {
    let mut parts = value.split(':');
    let hour = parts
        .next()
        .and_then(|x| u32::from_str(x).ok())
        .unwrap_or(2);
    let minute = parts
        .next()
        .and_then(|x| u32::from_str(x).ok())
        .unwrap_or(0);
    if hour > 23 || minute > 59 {
        (2, 0)
    } else {
        (hour, minute)
    }
}
