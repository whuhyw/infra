namespace InformationProvider.Models;

public record ElectricityBalanceResponse
{
    public string RoomId { get; init; } = "";
    public string RoomName { get; init; } = "";
    public string MeterId { get; init; } = "";
    public BalanceInfo Balance { get; init; } = new();
    public UsageInfo TotalUsage { get; init; } = new();
    public string UpdatedAt { get; init; } = "";
}
 
public record BalanceInfo
{
    public decimal Amount { get; init; }
    public string Unit { get; init; } = "元";
}

public record UsageInfo
{
    public decimal Amount { get; init; }
    public string Unit { get; init; } = "";
}
