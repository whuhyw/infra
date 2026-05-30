namespace InformationProvider.Services;

public interface ITokenService
{
    Task<string> GetAccessTokenAsync(CancellationToken ct = default);
}
