using InformationProvider.Models;

namespace InformationProvider.Services;

public class RoomService : IRoomService
{
    private readonly WhuApiHttpClient _client;

    public RoomService(WhuApiHttpClient client)
    {
        _client = client;
    }

    public async Task<List<AreaSummary>> GetAreasAsync(CancellationToken ct)
    {
        var result = await _client.GetDataOrNullAsync<WhuAreaData>(
            "/v3/XINTF/GetAreaInfo", null, ct);

        return result?.areaInfoList?.Select(a => new AreaSummary
        {
            AreaId = a.AreaID,
            AreaName = a.AreaName
        }).ToList() ?? [];
    }

    public async Task<List<ArchitectureSummary>> GetArchitecturesAsync(
        string areaId, CancellationToken ct)
    {
        var result = await _client.GetDataOrNullAsync<WhuArchitectureData>(
            "/v3/XINTF/GetArchitectureInfo", new { AreaID = areaId }, ct);

        if (result?.Result != 0) return [];

        return result.architectureInfoList?.Select(a => new ArchitectureSummary
        {
            ArchitectureId = a.ArchitectureID,
            ArchitectureName = a.ArchitectureName,
            StoryCount = a.ArchitectureStorys
        }).ToList() ?? [];
    }

    public async Task<List<RoomSummary>> GetRoomsAsync(
        string buildingId, int floor, CancellationToken ct)
    {
        var result = await _client.GetDataOrNullAsync<WhuRoomListData>(
            "/v3/XINTF/GetRoomInfo",
            new { ArchitectureID = buildingId, Floor = floor }, ct);

        return result?.roomInfoList?.Select(r => new RoomSummary
        {
            RoomNo = r.RoomNo,
            RoomName = r.RoomName
        }).ToList() ?? [];
    }
}
