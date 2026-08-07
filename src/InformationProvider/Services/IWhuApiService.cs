using InformationProvider.Models;

namespace InformationProvider.Services;

public interface IWhuApiService
{
    Task<RoomMeterInfo> GetRoomMeterInfoAsync(string roomId, CancellationToken ct = default);
    Task<ReserveInfo> GetReserveAsync(string meterId, CancellationToken ct = default);
    Task<IReadOnlyList<DayUsageItem>> GetMeterDayValueAsync(
        string meterId, DateOnly startDate, DateOnly endDate, CancellationToken ct = default);
}
