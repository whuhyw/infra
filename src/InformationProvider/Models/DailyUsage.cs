namespace InformationProvider.Models;

public record DailyUsageResponse
{
    public string RoomId { get; init; } = "";
    public string RoomName { get; init; } = "";
    public string MeterId { get; init; } = "";
    public string Date { get; init; } = "";
    public DailyUsageInfo Usage { get; init; } = new();
    public DailyCostInfo Cost { get; init; } = new();
    public string StartReading { get; init; } = "";
    public string EndReading { get; init; } = "";
}

public record DailyUsageInfo
{
    public decimal Amount { get; init; }
    public string Unit { get; init; } = "度";
}

public record DailyCostInfo
{
    public decimal Amount { get; init; }
    public string Unit { get; init; } = "元";
}
