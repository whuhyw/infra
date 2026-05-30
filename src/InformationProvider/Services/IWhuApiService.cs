using InformationProvider.Models;

namespace InformationProvider.Services;

public interface IWhuApiService
{
    Task<ElectricityBalanceResponse> GetBalanceAsync(string roomId, CancellationToken ct = default);
    Task<DailyUsageResponse> GetDailyUsageAsync(string roomId, DateTime date, CancellationToken ct = default);
}
