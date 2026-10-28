use serde::Serialize;

use crate::Result;
use crate::client::WhuApiClient;
use crate::models::{
    ArchitectureData, ArchitectureSummary, AreaData, AreaSummary, RoomListData, RoomSummary,
};

pub struct RoomService {
    client: WhuApiClient,
}

impl RoomService {
    pub fn new(client: WhuApiClient) -> Self {
        Self { client }
    }

    pub async fn get_areas(&self) -> Result<Vec<AreaSummary>> {
        let response: Option<AreaData> = self
            .client
            .get_data_or_null::<AreaData, EmptyQuery>("/v3/XINTF/GetAreaInfo", None)
            .await?;
        Ok(response
            .map(|data| {
                data.areas
                    .into_iter()
                    .map(|area| AreaSummary {
                        area_id: area.id,
                        area_name: area.name,
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn get_architectures(&self, area_id: &str) -> Result<Vec<ArchitectureSummary>> {
        let response: Option<ArchitectureData> = self
            .client
            .get_data_or_null(
                "/v3/XINTF/GetArchitectureInfo",
                Some(&AreaQuery { area_id }),
            )
            .await?;
        Ok(response
            .filter(|data| data.result == 0)
            .map(|data| {
                data.architectures
                    .into_iter()
                    .map(|item| ArchitectureSummary {
                        architecture_id: item.id,
                        architecture_name: item.name,
                        story_count: item.stories,
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn get_rooms(&self, building_id: &str, floor: i32) -> Result<Vec<RoomSummary>> {
        let response: Option<RoomListData> = self
            .client
            .get_data_or_null(
                "/v3/XINTF/GetRoomInfo",
                Some(&RoomQuery { building_id, floor }),
            )
            .await?;
        Ok(response
            .map(|data| {
                data.rooms
                    .into_iter()
                    .map(|room| RoomSummary {
                        room_no: room.number,
                        room_name: room.name,
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[derive(Serialize)]
struct EmptyQuery {}
#[derive(Serialize)]
struct AreaQuery<'a> {
    #[serde(rename = "AreaID")]
    area_id: &'a str,
}
#[derive(Serialize)]
struct RoomQuery<'a> {
    #[serde(rename = "ArchitectureID")]
    building_id: &'a str,
    #[serde(rename = "Floor")]
    floor: i32,
}
