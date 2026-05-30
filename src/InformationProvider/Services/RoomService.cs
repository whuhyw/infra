using System.Net.Http.Json;
using System.Reflection;
using Microsoft.Extensions.Options;
using InformationProvider.Configuration;
using InformationProvider.Models;

namespace InformationProvider.Services;

public class RoomService : IRoomService
{
    private readonly WhuApiOptions _options;
    private readonly ITokenService _tokenService;
    private readonly IHttpClientFactory _httpClientFactory;

    public RoomService(
        IOptions<WhuApiOptions> options,
        ITokenService tokenService,
        IHttpClientFactory httpClientFactory)
    {
        _options = options.Value;
        _tokenService = tokenService;
        _httpClientFactory = httpClientFactory;
    }

    public async Task<List<AreaSummary>> GetAreasAsync(CancellationToken ct)
    {
        var result = await CallAsync<WhuAreaData>(
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
        var result = await CallAsync<WhuArchitectureData>(
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
        var result = await CallAsync<WhuRoomListData>(
            "/v3/XINTF/GetRoomInfo",
            new { ArchitectureID = buildingId, Floor = floor }, ct);

        return result?.roomInfoList?.Select(r => new RoomSummary
        {
            RoomNo = r.RoomNo,
            RoomName = r.RoomName
        }).ToList() ?? [];
    }

    private async Task<T?> CallAsync<T>(
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

        return wrapper?.Data;
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
