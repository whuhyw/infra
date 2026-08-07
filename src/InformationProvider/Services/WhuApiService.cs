using InformationProvider.Models;

namespace InformationProvider.Services;

public class WhuApiService : IWhuApiService
{
    private readonly WhuApiHttpClient _client;

    public WhuApiService(WhuApiHttpClient client)
    {
        _client = client;
    }

    public async Task<RoomMeterInfo> GetRoomMeterInfoAsync(
        string roomId, CancellationToken ct = default)
    {
        var roomInfo = await _client.GetDataAsync<WhuRoomMeterData>(
            "/v3/XINTF/GetRoomMeterInfo", new { RoomID = roomId }, ct);

        if (roomInfo.Result != 0)
            throw new InvalidOperationException($"查询房间失败: {roomInfo.Msg}");

        var meter = roomInfo.meterList?.FirstOrDefault()
            ?? throw new InvalidOperationException("该房间未绑定电表");

        return new RoomMeterInfo
        {
            RoomId = roomId,
            RoomName = roomInfo.roomInfo?.RoomEntierName ?? "",
            MeterId = meter.meterId
        };
    }

    public async Task<ReserveInfo> GetReserveAsync(
        string meterId, CancellationToken ct = default)
    {
        var reserve = await _client.GetDataAsync<WhuReserveData>(
            "/v3/XINTF/GetReserve", new { MeterID = meterId }, ct);

        return new ReserveInfo
        {
            Balance = decimal.TryParse(reserve.remainPower, out var b) ? b : 0,
            CumulativeUsage = decimal.TryParse(reserve.ZVlaue, out var u) ? u : 0,
            ReadTime = reserve.readTime
        };
    }

    public async Task<IReadOnlyList<DayUsageItem>> GetMeterDayValueAsync(
        string meterId, DateOnly startDate, DateOnly endDate, CancellationToken ct = default)
    {
        var dayVal = await _client.GetDataAsync<WhuDayValueData>(
            "/v3/XINTF/GetMeterDayValue",
            new
            {
                MeterID = meterId,
                startDate = $"{startDate.Year}-{startDate.Month}-{startDate.Day}",
                endDate = $"{endDate.Year}-{endDate.Month}-{endDate.Day}"
            },
            ct);

        return dayVal.DayValues.Select(i => new DayUsageItem
        {
            Date = i.curDayTime,
            Usage = decimal.TryParse(i.dayValue, out var v) ? v : 0,
            Cost = decimal.TryParse(i.dayUseMeony, out var c) ? c : 0,
            StartReading = i.StarZValueZY ?? "",
            EndReading = i.EndZValueZY ?? ""
        }).ToList();
    }
}
