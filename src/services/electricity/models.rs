use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub struct RoomMeterInfo {
    pub room_id: String,
    pub room_name: String,
    pub meter_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReserveInfo {
    pub balance: f64,
    pub cumulative_usage: f64,
    pub read_time: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DayUsageItem {
    pub date: String,
    pub usage: f64,
    pub cost: f64,
    pub start_reading: String,
    pub end_reading: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaSummary {
    pub area_id: String,
    pub area_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchitectureSummary {
    pub architecture_id: String,
    pub architecture_name: String,
    pub story_count: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomSummary {
    pub room_no: String,
    pub room_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
pub(crate) struct WhuApiResponse<T> {
    #[serde(rename = "Code", alias = "code")]
    pub code: i32,
    #[serde(rename = "Mess", alias = "mess", default)]
    pub message: String,
    #[serde(rename = "Data", alias = "data", default)]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SignInData {
    #[serde(rename = "access_Jwt", default)]
    pub access_jwt: String,
    #[serde(rename = "refresh_Jwt", default)]
    #[allow(dead_code)]
    pub refresh_jwt: String,
    #[serde(rename = "Result", default)]
    pub result: i32,
    #[serde(rename = "Msg", default)]
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoomMeterData {
    #[serde(rename = "roomInfo", default)]
    pub room_info: Option<RoomInfoData>,
    #[serde(rename = "meterList", default)]
    pub meter_list: Vec<MeterItem>,
    #[serde(rename = "Result", default)]
    pub result: i32,
    #[serde(rename = "Msg", default)]
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoomInfoData {
    #[serde(rename = "RoomEntierName", default)]
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MeterItem {
    #[serde(rename = "meterId", default)]
    pub meter_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReserveData {
    #[serde(rename = "remainPower", default)]
    pub remain_power: String,
    #[serde(rename = "ZVlaue", default)]
    pub cumulative_usage: String,
    #[serde(rename = "readTime", default)]
    pub read_time: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DayUsageData {
    #[serde(rename = "DayValues", default)]
    pub day_values: Vec<DayValueRaw>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DayValueRaw {
    #[serde(rename = "dayValue", default)]
    pub day_value: String,
    #[serde(rename = "dayUseMeony", default)]
    pub day_use_money: String,
    #[serde(rename = "StarZValueZY", default)]
    pub start_reading: String,
    #[serde(rename = "EndZValueZY", default)]
    pub end_reading: String,
    #[serde(rename = "curDayTime", default)]
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AreaData {
    #[serde(rename = "areaInfoList", default)]
    pub areas: Vec<AreaRaw>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AreaRaw {
    #[serde(rename = "AreaID", default)]
    pub id: String,
    #[serde(rename = "AreaName", default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArchitectureData {
    #[serde(rename = "architectureInfoList", default)]
    pub architectures: Vec<ArchitectureRaw>,
    #[serde(rename = "Result", default)]
    pub result: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArchitectureRaw {
    #[serde(rename = "ArchitectureID", default)]
    pub id: String,
    #[serde(rename = "ArchitectureName", default)]
    pub name: String,
    #[serde(rename = "ArchitectureStorys", default)]
    pub stories: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoomListData {
    #[serde(rename = "roomInfoList", default)]
    pub rooms: Vec<RoomRaw>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoomRaw {
    #[serde(rename = "RoomNo", default)]
    pub number: String,
    #[serde(rename = "RoomName", default)]
    pub name: String,
}

pub(crate) fn parse_number(value: &str) -> f64 {
    value.parse().unwrap_or(0.0)
}

pub(crate) fn format_date(date: NaiveDate) -> String {
    format!("{}-{}-{}", date.year(), date.month(), date.day())
}

pub(crate) fn parse_api_date(value: &str) -> Option<NaiveDate> {
    [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
    ]
    .iter()
    .find_map(|format| NaiveDate::parse_from_str(value, format).ok())
    .or_else(|| {
        ["%Y-%m-%d %H:%M:%S", "%Y/%m/%d %H:%M:%S"]
            .iter()
            .find_map(|format| {
                chrono::NaiveDateTime::parse_from_str(value, format)
                    .ok()
                    .map(|dt| dt.date())
            })
    })
}

use chrono::Datelike;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_api_dates_without_zero_padding() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 8).unwrap();
        assert_eq!(format_date(date), "2026-7-8");
    }

    #[test]
    fn invalid_numeric_api_values_fall_back_to_zero() {
        assert_eq!(parse_number("not-a-number"), 0.0);
    }

    #[test]
    fn normalizes_whu_date_formats() {
        assert_eq!(
            parse_api_date("2026/07/08"),
            NaiveDate::from_ymd_opt(2026, 7, 8)
        );
        assert_eq!(
            parse_api_date("2026-07-08 12:30:00"),
            NaiveDate::from_ymd_opt(2026, 7, 8)
        );
    }
}
