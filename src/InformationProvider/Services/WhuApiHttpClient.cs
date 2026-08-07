using System.Net.Http.Json;
using InformationProvider.Models;

namespace InformationProvider.Services;

public class WhuApiHttpClient
{
    private readonly WhuApiTransport _transport;
    private readonly ITokenService _tokenService;

    public WhuApiHttpClient(WhuApiTransport transport, ITokenService tokenService)
    {
        _transport = transport;
        _tokenService = tokenService;
    }

    public async Task<T> GetDataAsync<T>(
        string path, object? queryParams, CancellationToken ct = default) where T : class
    {
        var wrapper = await GetWrapperAsync<T>(path, queryParams, ct);
        if (wrapper?.Code != 0 || wrapper.Data is null)
            throw new InvalidOperationException($"API error [{path}]: {wrapper?.Mess}");
        return wrapper.Data;
    }

    public async Task<T?> GetDataOrNullAsync<T>(
        string path, object? queryParams, CancellationToken ct = default) where T : class
    {
        var wrapper = await GetWrapperAsync<T>(path, queryParams, ct);
        return wrapper?.Data;
    }

    private async Task<WhuApiResponse<T>?> GetWrapperAsync<T>(
        string path, object? queryParams, CancellationToken ct) where T : class
    {
        var token = await _tokenService.GetAccessTokenAsync(ct);
        var url = _transport.BuildUrl(path, queryParams);

        using var client = _transport.CreateClient();
        client.DefaultRequestHeaders.Add("Authorization", $"Bearer {token}");

        var response = await client.GetAsync(url, ct);
        response.EnsureSuccessStatusCode();

        return await response.Content.ReadFromJsonAsync<WhuApiResponse<T>>(ct);
    }
}
