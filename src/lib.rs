mod services;

pub use services::electricity::{
    ArchitectureSummary, AreaSummary, DayUsageItem, Error, ReserveInfo, Result, RoomMeterInfo,
    RoomService, RoomSummary, WhuApiClient, WhuApiOptions, WhuApiService,
};

pub async fn run_server() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    services::electricity::run().await
}
