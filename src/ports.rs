use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    Result,
    models::{
        ArchitectureSummary, AreaSummary, DayUsageItem, ReserveInfo, RoomMeterInfo, RoomSummary,
    },
};

#[async_trait]
pub(crate) trait WhuGateway: Send + Sync {
    async fn get_room_meter_info(&self, room_id: &str) -> Result<RoomMeterInfo>;
    async fn get_reserve(&self, meter_id: &str) -> Result<ReserveInfo>;
    async fn get_meter_day_value(
        &self,
        meter_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DayUsageItem>>;
}

#[async_trait]
impl WhuGateway for crate::WhuApiService {
    async fn get_room_meter_info(&self, room_id: &str) -> Result<RoomMeterInfo> {
        crate::WhuApiService::get_room_meter_info(self, room_id).await
    }

    async fn get_reserve(&self, meter_id: &str) -> Result<ReserveInfo> {
        crate::WhuApiService::get_reserve(self, meter_id).await
    }

    async fn get_meter_day_value(
        &self,
        meter_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DayUsageItem>> {
        crate::WhuApiService::get_meter_day_value(self, meter_id, start_date, end_date).await
    }
}

#[async_trait]
pub(crate) trait RoomDirectory: Send + Sync {
    async fn get_areas(&self) -> Result<Vec<AreaSummary>>;
    async fn get_architectures(&self, area_id: &str) -> Result<Vec<ArchitectureSummary>>;
    async fn get_rooms(&self, building_id: &str, floor: i32) -> Result<Vec<RoomSummary>>;
}

#[async_trait]
impl RoomDirectory for crate::RoomService {
    async fn get_areas(&self) -> Result<Vec<AreaSummary>> {
        crate::RoomService::get_areas(self).await
    }

    async fn get_architectures(&self, area_id: &str) -> Result<Vec<ArchitectureSummary>> {
        crate::RoomService::get_architectures(self, area_id).await
    }

    async fn get_rooms(&self, building_id: &str, floor: i32) -> Result<Vec<RoomSummary>> {
        crate::RoomService::get_rooms(self, building_id, floor).await
    }
}
