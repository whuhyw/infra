using System.Net.Http.Json;
using System.Reflection;
using Microsoft.Extensions.Options;
using InformationProvider.Configuration;
using InformationProvider.Models;

namespace InformationProvider.Services;

public class WhuApiTransport
{
    private readonly WhuApiOptions _options;
    private readonly IHttpClientFactory _httpClientFactory;

    public WhuApiTransport(
        IOptions<WhuApiOptions> options,
        IHttpClientFactory httpClientFactory)
    {
        _options = options.Value;
        _httpClientFactory = httpClientFactory;
    }

    public string BuildUrl(string path, object? queryParams)
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

    public HttpClient CreateClient()
    {
        var client = _httpClientFactory.CreateClient();
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
        return client;
    }

    public async Task<WhuApiResponse<T>?> GetRawAsync<T>(
        string url, CancellationToken ct = default) where T : class
    {
        using var client = CreateClient();
        return await client.GetFromJsonAsync<WhuApiResponse<T>>(url, ct);
    }
}
