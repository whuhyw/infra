using System.Net.Http.Json;
using System.Reflection;
using System.Text;
using Microsoft.Extensions.Options;
using InformationProvider.Configuration;
using InformationProvider.Models;

namespace InformationProvider.Services;

public class WhuApiService : IWhuApiService
{
    private readonly WhuApiOptions _options;
    private readonly ITokenService _tokenService;
    private readonly IHttpClientFactory _httpClientFactory;
    private readonly ILogger<WhuApiService> _logger;

    public WhuApiService(
        IOptions<WhuApiOptions> options,
        ITokenService tokenService,
        IHttpClientFactory httpClientFactory,
        ILogger<WhuApiService> logger)
    {
        _options = options.Value;
        _tokenService = tokenService;
        _httpClientFactory = httpClientFactory;
        _logger = logger;
    }

    public async Task<ElectricityBalanceResponse> GetBalanceAsync(
        string roomId, CancellationToken ct = default)
    {
        var (roomName, meterId) = await GetMeterIdAsync(roomId, ct);

        var reserve = await CallAsync<WhuReserveData>(
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

        var dayVal = await CallAsync<WhuDayValueData>(
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
        var roomInfo = await CallAsync<WhuRoomMeterData>(
            "/v3/XINTF/GetRoomMeterInfo", new { RoomID = roomId }, ct);

        if (roomInfo.Result != 0)
            throw new InvalidOperationException(
                $"查询房间失败: {roomInfo.Msg}");

        var meter = roomInfo.meterList?.FirstOrDefault()
            ?? throw new InvalidOperationException("该房间未绑定电表");

        return (roomInfo.roomInfo?.RoomEntierName ?? "", meter.meterId);
    }

    private async Task<T> CallAsync<T>(
        string path, object? queryParams, CancellationToken ct) where T : class
    {
        var token = await _tokenService.GetAccessTokenAsync(ct);
        var url = BuildUrl(path, queryParams);

        var client = _httpClientFactory.CreateClient();
        SetDefaultHeaders(client);
        client.DefaultRequestHeaders.Add("Authorization", $"Bearer {token}");

        var response = await client.GetAsync(url, ct);
        response.EnsureSuccessStatusCode();

        var wrapper = await response.Content
            .ReadFromJsonAsync<WhuApiResponse<T>>(ct);

        if (wrapper?.Code != 0 || wrapper.Data is null)
            throw new InvalidOperationException(
                $"API error [{path}]: {wrapper?.Mess}");

        return wrapper.Data;
    }

    private string BuildUrl(string path, object? queryParams)
    {
        var url = $"{_options.BaseUrl}{path}";
        if (queryParams is null) return url;

        var pairs = new List<string>();
        foreach (var prop in queryParams.GetType().GetProperties(
                     BindingFlags.Public | BindingFlags.Instance))
        {
            var value = prop.GetValue(queryParams)?.ToString();
            if (value is not null)
                pairs.Add($"{prop.Name}={Uri.EscapeDataString(value)}");
        }

        return url + "?" + string.Join("&", pairs);
    }

    private static void SetDefaultHeaders(HttpClient client)
    {
        client.DefaultRequestHeaders.TryAddWithoutValidation("User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
        client.DefaultRequestHeaders.TryAddWithoutValidation("Accept",
            "application/json, text/plain, */*");
        client.DefaultRequestHeaders.TryAddWithoutValidation("Accept-Language",
            "zh-CN,zh;q=0.9");
        client.DefaultRequestHeaders.TryAddWithoutValidation("Referer",
            "http://zwhqbsd.whu.edu.cn/MobilePayWeb/?t=20260123");
        client.DefaultRequestHeaders.TryAddWithoutValidation("Origin",
            "http://zwhqbsd.whu.edu.cn");
    }
}
