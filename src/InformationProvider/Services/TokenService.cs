using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Microsoft.Extensions.Options;
using InformationProvider.Configuration;
using InformationProvider.Models;

namespace InformationProvider.Services;

public class TokenService : ITokenService, IDisposable
{
    private readonly WhuApiOptions _options;
    private readonly ISm2CryptoService _sm2;
    private readonly WhuApiTransport _transport;
    private readonly ILogger<TokenService> _logger;

    private string? _accessToken;
    private string? _refreshToken;
    private DateTime _expiresAt = DateTime.MinValue;
    private readonly SemaphoreSlim _lock = new(1, 1);

    public TokenService(
        IOptions<WhuApiOptions> options,
        ISm2CryptoService sm2Crypto,
        WhuApiTransport transport,
        ILogger<TokenService> logger)
    {
        _options = options.Value;
        _sm2 = sm2Crypto;
        _transport = transport;
        _logger = logger;
    }

    public async Task<string> GetAccessTokenAsync(CancellationToken ct = default)
    {
        if (!string.IsNullOrEmpty(_accessToken) && DateTime.UtcNow < _expiresAt.AddMinutes(-10))
            return _accessToken;

        await _lock.WaitAsync(ct);
        try
        {
            if (!string.IsNullOrEmpty(_accessToken) && DateTime.UtcNow < _expiresAt.AddMinutes(-10))
                return _accessToken;

            await RefreshTokenAsync(ct);
            return _accessToken!;
        }
        finally
        {
            _lock.Release();
        }
    }

    private async Task RefreshTokenAsync(CancellationToken ct)
    {
        var ts = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        var p2 = Md5Hex(_options.AccountPass);
        var raw = $"p1={_options.AccountId}&p2={p2}&p3={_options.SysId}&t={ts}";
        var s = _sm2.Encrypt(raw);

        var url = $"{_options.BaseUrl}/v3/XINTFLg/SpecialSignIn" +
                  $"?p1={_options.AccountId}&p2={p2}&p3={_options.SysId}&t={ts}&s={s}";

        var response = await _transport.GetRawAsync<WhuSignInData>(url, ct);

        if (response?.Code != 0 || response.Data?.Result != 0)
        {
            var msg = response?.Data?.Msg ?? "unknown";
            _logger.LogError("Token refresh failed: {Msg}", msg);
            throw new InvalidOperationException($"Token refresh failed: {msg}");
        }

        _accessToken = response.Data.access_Jwt;
        _refreshToken = response.Data.refresh_Jwt;
        _expiresAt = ParseJwtExpiry(_accessToken);

        _logger.LogInformation("Token obtained, expires at {Expires:O}", _expiresAt);
    }

    private static DateTime ParseJwtExpiry(string jwt)
    {
        try
        {
            var parts = jwt.Split('.');
            if (parts.Length < 2) return DateTime.MinValue;

            var payload = parts[1]
                .Replace('-', '+')
                .Replace('_', '/');
            var pad = payload.Length % 4;
            if (pad == 2) payload += "==";
            else if (pad == 3) payload += "=";

            var json = Encoding.UTF8.GetString(Convert.FromBase64String(payload));
            using var doc = JsonDocument.Parse(json);
            if (doc.RootElement.TryGetProperty("exp", out var expProp))
                return DateTimeOffset.FromUnixTimeSeconds(expProp.GetInt64()).UtcDateTime;
        }
        catch
        {
            // ignore parse errors
        }
        return DateTime.MinValue;
    }

    private static string Md5Hex(string input)
    {
        var bytes = MD5.HashData(Encoding.UTF8.GetBytes(input));
        return Convert.ToHexString(bytes);
    }

    public void Dispose() => _lock.Dispose();
}
