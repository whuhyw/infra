use chrono::NaiveDate;
use serde::Serialize;

use crate::client::WhuApiClient;
use crate::models::{
    DayUsageData, DayUsageItem, ReserveData, ReserveInfo, RoomMeterData, RoomMeterInfo,
    format_date, parse_number,
};
use crate::{Error, Result};

pub struct WhuApiService {
    client: WhuApiClient,
}

impl WhuApiService {
    pub fn new(client: WhuApiClient) -> Self {
        Self { client }
    }

    pub async fn get_room_meter_info(&self, room_id: &str) -> Result<RoomMeterInfo> {
        let query = RoomIdQuery { room_id };
        let response: RoomMeterData = self
            .client
            .get_data("/v3/XINTF/GetRoomMeterInfo", Some(&query))
            .await?;
        if response.result != 0 {
            return Err(Error::InvalidData(format!(
                "查询房间失败: {}",
                response.message
            )));
        }
        let meter_id = response
            .meter_list
            .first()
            .map(|meter| meter.meter_id.clone())
            .filter(|id| !id.is_empty())
            .ok_or_else(|| Error::InvalidData("该房间未绑定电表".into()))?;
        Ok(RoomMeterInfo {
            room_id: room_id.to_owned(),
            room_name: response
                .room_info
                .map(|room| room.full_name)
                .unwrap_or_default(),
            meter_id,
        })
    }

    pub async fn get_reserve(&self, meter_id: &str) -> Result<ReserveInfo> {
        let response: ReserveData = self
            .client
            .get_data("/v3/XINTF/GetReserve", Some(&MeterQuery { meter_id }))
            .await?;
        Ok(ReserveInfo {
            balance: parse_number(&response.remain_power),
            cumulative_usage: parse_number(&response.cumulative_usage),
            read_time: response.read_time,
        })
    }

    pub async fn get_meter_day_value(
        &self,
        meter_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DayUsageItem>> {
        let query = DayValueQuery {
            meter_id,
            start_date: format_date(start_date),
            end_date: format_date(end_date),
        };
        let response: DayUsageData = self
            .client
            .get_data("/v3/XINTF/GetMeterDayValue", Some(&query))
            .await?;
        Ok(response
            .day_values
            .into_iter()
            .map(|item| DayUsageItem {
                date: item.date,
                usage: parse_number(&item.day_value),
                cost: parse_number(&item.day_use_money),
                start_reading: item.start_reading,
                end_reading: item.end_reading,
            })
            .collect())
    }
}

#[derive(Serialize)]
struct RoomIdQuery<'a> {
    #[serde(rename = "RoomID")]
    room_id: &'a str,
}
#[derive(Serialize)]
struct MeterQuery<'a> {
    #[serde(rename = "MeterID")]
    meter_id: &'a str,
}
#[derive(Serialize)]
struct DayValueQuery<'a> {
    #[serde(rename = "MeterID")]
    meter_id: &'a str,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}
