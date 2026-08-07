using InformationProvider.Models;

namespace InformationProvider.Services;

public class WhuApiService : IWhuApiService
{
    private readonly WhuApiHttpClient _client;

    public WhuApiService(WhuApiHttpClient client)
    {
        _client = client;
    }

    public async Task<ElectricityBalanceResponse> GetBalanceAsync(
        string roomId, CancellationToken ct = default)
    {
        var (roomName, meterId) = await GetMeterIdAsync(roomId, ct);

        var reserve = await _client.GetDataAsync<WhuReserveData>(
            "/v3/XINTF/GetReserve", new { MeterID = meterId }, ct);

        return new ElectricityBalanceResponse
        {
            RoomId = roomId,
            RoomName = roomName,
            MeterId = meterId,
            Balance = new BalanceInfo
            {
                Amount = decimal.TryParse(reserve.remainPower, out var b) ? b : 0,
                Unit = string.IsNullOrEmpty(reserve.remainName) ? "元" : reserve.remainName
            },
            TotalUsage = new UsageInfo
            {
                Amount = decimal.TryParse(reserve.ZVlaue, out var u) ? u : 0,
                Unit = string.IsNullOrEmpty(reserve.unit) ? "度" : reserve.unit
            },
            UpdatedAt = reserve.readTime
        };
    }

    public async Task<DailyUsageResponse> GetDailyUsageAsync(
        string roomId, DateTime date, CancellationToken ct = default)
    {
        var (roomName, meterId) = await GetMeterIdAsync(roomId, ct);
        var dateStr = $"{date.Year}-{date.Month}-{date.Day}";

        var dayVal = await _client.GetDataAsync<WhuDayValueData>(
            "/v3/XINTF/GetMeterDayValue",
            new { MeterID = meterId, startDate = dateStr, endDate = dateStr },
            ct);

        var item = dayVal.DayValues?.FirstOrDefault();

        return new DailyUsageResponse
        {
            RoomId = roomId,
            RoomName = roomName,
            MeterId = meterId,
            Date = dateStr,
            Usage = new DailyUsageInfo
            {
                Amount = item is not null && decimal.TryParse(item.dayValue, out var v) ? v : 0,
                Unit = string.IsNullOrEmpty(item?.dw) ? "度" : item.dw
            },
            Cost = new DailyCostInfo
            {
                Amount = item is not null && decimal.TryParse(item.dayUseMeony, out var c) ? c : 0,
                Unit = "元"
            },
            StartReading = item?.StarZValueZY ?? "",
            EndReading = item?.EndZValueZY ?? ""
        };
    }

    private async Task<(string roomName, string meterId)> GetMeterIdAsync(
        string roomId, CancellationToken ct)
    {
        var roomInfo = await _client.GetDataAsync<WhuRoomMeterData>(
            "/v3/XINTF/GetRoomMeterInfo", new { RoomID = roomId }, ct);

        if (roomInfo.Result != 0)
            throw new InvalidOperationException(
                $"查询房间失败: {roomInfo.Msg}");

        var meter = roomInfo.meterList?.FirstOrDefault()
            ?? throw new InvalidOperationException("该房间未绑定电表");

        return (roomInfo.roomInfo?.RoomEntierName ?? "", meter.meterId);
    }
}
