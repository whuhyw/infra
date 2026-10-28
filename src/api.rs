use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header::LOCATION},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{NaiveDate, Utc};
use chrono_tz::Asia::Shanghai;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock, Semaphore};

use crate::{
    config::AppConfig,
    models::parse_api_date,
    ports::{RoomDirectory, WhuGateway},
    store::{BackfillJob, Store},
};

#[derive(Clone)]
pub(crate) struct AppState {
    pub store: Store,
    pub whu: Arc<dyn WhuGateway>,
    pub rooms: Arc<dyn RoomDirectory>,
    pub config: AppConfig,
    query_limits: Arc<Mutex<HashMap<String, Instant>>>,
    refresh_limits: Arc<Mutex<HashMap<String, Instant>>>,
    pub last_scheduled_run: Arc<RwLock<Option<String>>>,
    pub backfill_semaphore: Arc<Semaphore>,
}

impl AppState {
    pub(crate) fn new(
        store: Store,
        whu: Arc<dyn WhuGateway>,
        rooms: Arc<dyn RoomDirectory>,
        config: AppConfig,
    ) -> Self {
        let max_concurrent = config.max_concurrent.max(1);
        Self {
            store,
            whu,
            rooms,
            config,
            query_limits: Arc::new(Mutex::new(HashMap::new())),
            refresh_limits: Arc::new(Mutex::new(HashMap::new())),
            last_scheduled_run: Arc::new(RwLock::new(None)),
            backfill_semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    async fn limited(
        &self,
        map: &Mutex<HashMap<String, Instant>>,
        key: String,
        interval: Duration,
    ) -> bool {
        let now = Instant::now();
        let mut limits = map.lock().await;
        limits.retain(|_, previous| now.duration_since(*previous) < interval.saturating_mul(2));
        if limits
            .get(&key)
            .is_some_and(|previous| now.duration_since(*previous) < interval)
        {
            return true;
        }
        limits.insert(key, now);
        false
    }

    async fn query_limit(&self, room: &str, category: &str) -> bool {
        let limited = self
            .limited(
                &self.query_limits,
                format!("{room}:{category}"),
                self.config.query_interval,
            )
            .await;
        if limited {
            tracing::warn!(category, "query rate limit triggered");
        }
        limited
    }

    async fn refresh_limit(&self, room: &str) -> bool {
        let limited = self
            .limited(
                &self.refresh_limits,
                room.to_owned(),
                self.config.refresh_interval,
            )
            .await;
        if limited {
            tracing::warn!("refresh rate limit triggered");
        }
        limited
    }
}

pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/areas", get(areas))
        .route("/api/areas/{area_id}/buildings", get(buildings))
        .route("/api/buildings/{building_id}/rooms", get(rooms))
        .route("/api/rooms", post(subscribe))
        .route("/api/rooms/{room_id}/backfill", get(backfill_status))
        .route("/api/rooms/{room_id}/balance", get(balance))
        .route("/api/rooms/{room_id}/daily-usage", get(daily_usage))
        .route("/api/rooms/{room_id}/trend", get(trend))
        .route("/api/rooms/{room_id}/refresh", post(refresh))
        .with_state(state)
}

#[derive(Debug)]
struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorBody { error: self.1 })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn internal(error: crate::Error) -> ApiError {
    tracing::error!(error = %error, "request failed");
    let status = match error {
        crate::Error::Database(_) | crate::Error::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_GATEWAY,
    };
    ApiError(status, "上游服务或数据库暂时不可用".into())
}

fn invalid_query(message: &str) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, message.into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AreaDto {
    area_id: String,
    area_name: String,
}

async fn areas(State(state): State<AppState>) -> Result<Json<Vec<AreaDto>>, ApiError> {
    state
        .rooms
        .get_areas()
        .await
        .map(|items| {
            Json(
                items
                    .into_iter()
                    .map(|area| AreaDto {
                        area_id: area.area_id,
                        area_name: area.area_name,
                    })
                    .collect(),
            )
        })
        .map_err(internal)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildingDto {
    architecture_id: String,
    architecture_name: String,
    story_count: i32,
}

async fn buildings(
    State(state): State<AppState>,
    Path(area_id): Path<String>,
) -> Result<Json<Vec<BuildingDto>>, ApiError> {
    state
        .rooms
        .get_architectures(&area_id)
        .await
        .map(|items| {
            Json(
                items
                    .into_iter()
                    .map(|building| BuildingDto {
                        architecture_id: building.architecture_id,
                        architecture_name: building.architecture_name,
                        story_count: building.story_count,
                    })
                    .collect(),
            )
        })
        .map_err(internal)
}

#[derive(Deserialize)]
struct FloorQuery {
    floor: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomDto {
    room_no: String,
    room_name: String,
}

async fn rooms(
    State(state): State<AppState>,
    Path(building_id): Path<String>,
    query: Result<Query<FloorQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<Vec<RoomDto>>, ApiError> {
    let Query(query) = query.map_err(|_| invalid_query("floor 必须是正整数"))?;
    if query.floor < 1 {
        return Err(ApiError(StatusCode::BAD_REQUEST, "楼层号必须大于 0".into()));
    }
    state
        .rooms
        .get_rooms(&building_id, query.floor)
        .await
        .map(|items| {
            Json(
                items
                    .into_iter()
                    .map(|room| RoomDto {
                        room_no: room.room_no,
                        room_name: room.room_name,
                    })
                    .collect(),
            )
        })
        .map_err(internal)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeRequest {
    room_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionResponse {
    room_id: String,
    room_name: String,
    subscribed_at: String,
    balance: f64,
    backfill_status: String,
}

async fn subscribe(
    State(state): State<AppState>,
    body: Result<Json<SubscribeRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(request) = body.map_err(|_| invalid_query("请求体必须是有效 JSON"))?;
    let room_id = request.room_id.trim();
    if room_id.is_empty() || !room_id.chars().all(|character| character.is_ascii_digit()) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "roomId 必须是数字字符串".into(),
        ));
    }
    if state
        .store
        .get_subscription(room_id)
        .await
        .map_err(internal)?
        .is_some()
    {
        return Err(ApiError(StatusCode::CONFLICT, "该房间已订阅".into()));
    }
    let info = state
        .whu
        .get_room_meter_info(room_id)
        .await
        .map_err(internal)?;
    let reserve = state
        .whu
        .get_reserve(&info.meter_id)
        .await
        .map_err(internal)?;
    let inserted = state
        .store
        .insert_subscription(
            room_id,
            &info.room_name,
            &info.meter_id,
            reserve.balance,
            &today().to_string(),
        )
        .await
        .map_err(internal)?;
    if !inserted {
        return Err(ApiError(StatusCode::CONFLICT, "该房间已订阅".into()));
    }
    let subscription = state
        .store
        .get_subscription(room_id)
        .await
        .map_err(internal)?
        .expect("inserted subscription must be readable");
    let job = state
        .store
        .backfill_job(room_id)
        .await
        .map_err(internal)?
        .expect("job inserted atomically");
    spawn_backfill(state.clone(), room_id.to_owned());

    let response = SubscriptionResponse {
        room_id: room_id.to_owned(),
        room_name: subscription.room_name,
        subscribed_at: subscription.subscribed_at,
        balance: reserve.balance,
        backfill_status: job.status,
    };
    let mut response = (StatusCode::ACCEPTED, Json(response)).into_response();
    response.headers_mut().insert(
        LOCATION,
        HeaderValue::from_str(&format!("/api/rooms/{room_id}/backfill")).unwrap(),
    );
    Ok(response)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackfillStatusDto {
    room_id: String,
    status: String,
    started_at: Option<String>,
    updated_at: String,
    error: Option<String>,
}

fn backfill_dto(job: BackfillJob) -> BackfillStatusDto {
    BackfillStatusDto {
        room_id: job.room_id,
        status: job.status,
        started_at: job.started_at,
        updated_at: job.updated_at,
        error: job.error,
    }
}

async fn backfill_status(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<BackfillStatusDto>, ApiError> {
    match state.store.backfill_job(&room_id).await.map_err(internal)? {
        Some(job) => Ok(Json(backfill_dto(job))),
        None => Err(ApiError(
            StatusCode::NOT_FOUND,
            "未找到该房间的回填任务".into(),
        )),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BalanceDto {
    room_id: String,
    balance: f64,
    date: String,
}

async fn balance(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<BalanceDto>, ApiError> {
    if state.query_limit(&room_id, "balance").await {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "查询过于频繁，请稍后重试".into(),
        ));
    }
    match state
        .store
        .latest_balance(&room_id)
        .await
        .map_err(internal)?
    {
        Some(row) => Ok(Json(BalanceDto {
            room_id: row.room_id,
            balance: row.balance,
            date: row.date,
        })),
        None => Err(ApiError(
            StatusCode::NOT_FOUND,
            "暂无该房间的余额数据".into(),
        )),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateQuery {
    date: String,
}

fn parse_date(value: &str) -> Result<NaiveDate, ApiError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .filter(|date| date.format("%Y-%m-%d").to_string() == value)
        .ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "日期格式必须为 yyyy-MM-dd".into()))
}

fn today() -> NaiveDate {
    Utc::now().with_timezone(&Shanghai).date_naive()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UsageDto {
    room_id: String,
    date: String,
    usage: f64,
    cost: f64,
}

async fn daily_usage(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    query: Result<Query<DateQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<UsageDto>, ApiError> {
    let Query(query) = query.map_err(|_| invalid_query("必须提供 date 参数"))?;
    if state.query_limit(&room_id, "daily-usage").await {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "查询过于频繁，请稍后重试".into(),
        ));
    }
    let date = parse_date(&query.date)?;
    if date > today() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "不能查询未来日期".into()));
    }
    match state
        .store
        .usage_for_date(&room_id, date)
        .await
        .map_err(internal)?
    {
        Some(row) => Ok(Json(UsageDto {
            room_id: row.room_id,
            date: row.date,
            usage: row.usage,
            cost: row.cost,
        })),
        None => Err(ApiError(
            StatusCode::NOT_FOUND,
            "暂无该日期的用量数据".into(),
        )),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrendQuery {
    start_date: String,
    end_date: String,
}

#[derive(Serialize)]
struct TrendValue {
    date: String,
    value: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrendDto {
    room_id: String,
    start_date: String,
    end_date: String,
    usage: Vec<TrendValue>,
    cost: Vec<TrendValue>,
    balance: Vec<TrendValue>,
}

async fn trend(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    query: Result<Query<TrendQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<TrendDto>, ApiError> {
    let Query(query) = query.map_err(|_| invalid_query("必须提供 startDate 和 endDate 参数"))?;
    if state.query_limit(&room_id, "trend").await {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "查询过于频繁，请稍后重试".into(),
        ));
    }
    let start = parse_date(&query.start_date)?;
    let end = parse_date(&query.end_date)?;
    if start > end {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "startDate 不能晚于 endDate".into(),
        ));
    }
    let (usage, cost, balance) = tokio::try_join!(
        state
            .store
            .trend("daily_usage_records", "usage", &room_id, start, end),
        state
            .store
            .trend("daily_usage_records", "cost", &room_id, start, end),
        state
            .store
            .trend("daily_balance_records", "balance", &room_id, start, end),
    )
    .map_err(internal)?;
    let convert = |rows: Vec<crate::store::TrendRow>| {
        rows.into_iter()
            .map(|row| TrendValue {
                date: row.date,
                value: row.value,
            })
            .collect()
    };
    Ok(Json(TrendDto {
        room_id,
        start_date: start.to_string(),
        end_date: end.to_string(),
        usage: convert(usage),
        cost: convert(cost),
        balance: convert(balance),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshDto {
    room_id: String,
    balance: f64,
    usage: Option<UsageValue>,
}

#[derive(Serialize)]
struct UsageValue {
    date: String,
    usage: f64,
    cost: f64,
}

async fn refresh(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<RefreshDto>, ApiError> {
    if !state.config.refresh_enabled {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "刷新功能已关闭".into(),
        ));
    }
    if state.refresh_limit(&room_id).await {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "刷新过于频繁，请稍后重试".into(),
        ));
    }
    let subscription = state
        .store
        .get_subscription(&room_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "该房间尚未订阅".into()))?;
    let (reserve, usage_items) = tokio::try_join!(
        state.whu.get_reserve(&subscription.meter_id),
        state
            .whu
            .get_meter_day_value(&subscription.meter_id, today(), today()),
    )
    .map_err(internal)?;
    let date = today().to_string();
    state
        .store
        .upsert_balance(&room_id, &date, reserve.balance)
        .await
        .map_err(internal)?;
    let usage = match usage_items.into_iter().next() {
        Some(item) => Some(UsageValue {
            date: parse_api_date(&item.date)
                .ok_or_else(|| {
                    internal(crate::Error::InvalidData(
                        "WHU returned an invalid usage date".into(),
                    ))
                })?
                .to_string(),
            usage: item.usage,
            cost: item.cost,
        }),
        None => None,
    };
    if let Some(item) = &usage {
        state
            .store
            .upsert_usage(&room_id, &item.date, item.usage, item.cost)
            .await
            .map_err(internal)?;
    }
    Ok(Json(RefreshDto {
        room_id,
        balance: reserve.balance,
        usage,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthDto {
    status: &'static str,
    db_connected: bool,
    last_scheduled_run: Option<String>,
    subscribed_rooms: i64,
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthDto>, ApiError> {
    let count = state.store.room_count().await.map_err(internal)?;
    let last = state.last_scheduled_run.read().await.clone();
    Ok(Json(HealthDto {
        status: "healthy",
        db_connected: true,
        last_scheduled_run: last,
        subscribed_rooms: count,
    }))
}

pub(crate) fn spawn_backfill(state: AppState, room_id: String) {
    tokio::spawn(async move {
        let permit = match state.backfill_semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(error) => {
                tracing::error!(%room_id, %error, "backfill semaphore closed");
                return;
            }
        };
        let result = run_backfill(&state, &room_id).await;
        drop(permit);
        match result {
            Ok(()) => {
                let _ = state
                    .store
                    .set_backfill_status(&room_id, "completed", None)
                    .await;
            }
            Err(error) => {
                tracing::error!(%room_id, %error, "backfill failed");
                let message = error.to_string();
                let _ = state
                    .store
                    .set_backfill_status(&room_id, "failed", Some(&message))
                    .await;
            }
        }
    });
}

async fn run_backfill(state: &AppState, room_id: &str) -> crate::Result<()> {
    state
        .store
        .set_backfill_status(room_id, "running", None)
        .await?;
    let subscription = state
        .store
        .get_subscription(room_id)
        .await?
        .ok_or_else(|| crate::Error::InvalidData("subscription disappeared".into()))?;
    let mut chunk_end = today().pred_opt().unwrap_or_else(today);
    loop {
        let chunk_start =
            chunk_end - chrono::Duration::days(state.config.backfill_chunk_days.max(1) - 1);
        let values = state
            .whu
            .get_meter_day_value(&subscription.meter_id, chunk_start, chunk_end)
            .await?;
        if values.is_empty() {
            break;
        }
        let mut oldest: Option<NaiveDate> = None;
        for item in values {
            let date = parse_api_date(&item.date).ok_or_else(|| {
                crate::Error::InvalidData(format!("WHU returned an invalid date: {}", item.date))
            })?;
            oldest = Some(oldest.map_or(date, |value| value.min(date)));
            state
                .store
                .upsert_usage(room_id, &date.to_string(), item.usage, item.cost)
                .await?;
        }
        let Some(oldest) = oldest else { break };
        let Some(previous) = oldest.pred_opt() else {
            break;
        };
        if previous >= chunk_end {
            break;
        }
        chunk_end = previous;
    }
    let reserve = state.whu.get_reserve(&subscription.meter_id).await?;
    state
        .store
        .upsert_balance(room_id, &today().to_string(), reserve.balance)
        .await?;
    Ok(())
}

pub(crate) async fn restart_pending_backfills(state: AppState) {
    match state.store.pending_backfills().await {
        Ok(jobs) => {
            for job in jobs {
                spawn_backfill(state.clone(), job.room_id);
            }
        }
        Err(error) => tracing::error!(%error, "failed to resume backfills"),
    }
}

pub(crate) fn scheduled_today() -> NaiveDate {
    Utc::now().with_timezone(&Shanghai).date_naive()
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use chrono::NaiveDate;
    use chrono::Utc;
    use tower::ServiceExt;

    use super::{AppState, router, today};
    use crate::{
        WhuApiOptions,
        config::AppConfig,
        models::{
            ArchitectureSummary, AreaSummary, DayUsageItem, ReserveInfo, RoomMeterInfo, RoomSummary,
        },
        ports::{RoomDirectory, WhuGateway},
        store::Store,
    };

    struct FakeWhu;

    #[async_trait::async_trait]
    impl WhuGateway for FakeWhu {
        async fn get_room_meter_info(&self, room_id: &str) -> crate::Result<RoomMeterInfo> {
            Ok(RoomMeterInfo {
                room_id: room_id.into(),
                room_name: "test room".into(),
                meter_id: "meter".into(),
            })
        }
        async fn get_reserve(&self, _: &str) -> crate::Result<ReserveInfo> {
            Ok(ReserveInfo {
                balance: 42.5,
                cumulative_usage: 10.0,
                read_time: String::new(),
            })
        }
        async fn get_meter_day_value(
            &self,
            _: &str,
            _: NaiveDate,
            _: NaiveDate,
        ) -> crate::Result<Vec<DayUsageItem>> {
            Ok(Vec::new())
        }
    }

    struct FakeRooms;

    #[async_trait::async_trait]
    impl RoomDirectory for FakeRooms {
        async fn get_areas(&self) -> crate::Result<Vec<AreaSummary>> {
            Ok(Vec::new())
        }
        async fn get_architectures(&self, _: &str) -> crate::Result<Vec<ArchitectureSummary>> {
            Ok(Vec::new())
        }
        async fn get_rooms(&self, _: &str, _: i32) -> crate::Result<Vec<RoomSummary>> {
            Ok(Vec::new())
        }
    }

    async fn test_app() -> (axum::Router, Store) {
        let store = Store::connect("sqlite::memory:").await.unwrap();
        let state = AppState::new(
            store.clone(),
            Arc::new(FakeWhu),
            Arc::new(FakeRooms),
            AppConfig {
                bind_address: "127.0.0.1:0".into(),
                database_url: "sqlite::memory:".into(),
                whu: WhuApiOptions::default(),
                query_interval: Duration::from_secs(3600),
                refresh_interval: Duration::from_secs(1800),
                refresh_enabled: true,
                scheduled_time: (2, 0),
                retry_interval: Duration::from_secs(900),
                max_concurrent: 5,
                backfill_chunk_days: 100,
            },
        );
        (router(state), store)
    }

    #[tokio::test]
    async fn balance_query_reads_database_and_enforces_per_type_limit() {
        let (app, store) = test_app().await;
        store
            .insert_subscription("314", "room", "meter", 42.5, &today().to_string())
            .await
            .unwrap();
        let date = Utc::now().date_naive().to_string();
        store.upsert_balance("314", &date, 42.5).await.unwrap();

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/rooms/314/balance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let body = to_bytes(first.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["balance"],
            42.5
        );

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/api/rooms/314/balance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn malformed_query_uses_standard_error_body() {
        let (app, _) = test_app().await;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/buildings/1/rooms?floor=x")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["error"].is_string());
    }

    #[tokio::test]
    async fn empty_trend_returns_three_empty_series() {
        let (app, _) = test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/rooms/314/trend?startDate=2026-07-01&endDate=2026-07-07")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["usage"].as_array().unwrap().len(), 0);
        assert_eq!(json["cost"].as_array().unwrap().len(), 0);
        assert_eq!(json["balance"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn future_usage_date_is_rejected() {
        let (app, _) = test_app().await;
        let future = (Utc::now().date_naive() + chrono::Duration::days(1)).to_string();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/rooms/314/daily-usage?date={future}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn refresh_updates_database_from_whu_values() {
        let (app, store) = test_app().await;
        store
            .insert_subscription("314", "room", "meter", 42.5, &today().to_string())
            .await
            .unwrap();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/rooms/314/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let row = store.latest_balance("314").await.unwrap().unwrap();
        assert_eq!(row.balance, 42.5);
    }

    #[tokio::test]
    async fn subscription_returns_accepted_and_persists_backfill_job() {
        let (app, store) = test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/rooms")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"roomId":"00123"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(response.headers()["location"], "/api/rooms/00123/backfill");
        let job = store.backfill_job("00123").await.unwrap().unwrap();
        assert!(["pending", "running", "completed"].contains(&job.status.as_str()));
        for _ in 0..50 {
            if store.backfill_job("00123").await.unwrap().unwrap().status == "completed" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        assert_eq!(
            store.backfill_job("00123").await.unwrap().unwrap().status,
            "completed"
        );
        assert_eq!(
            store
                .latest_balance("00123")
                .await
                .unwrap()
                .unwrap()
                .balance,
            42.5
        );
    }
}
