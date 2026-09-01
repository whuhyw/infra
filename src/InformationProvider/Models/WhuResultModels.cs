namespace InformationProvider.Models;

public record RoomMeterInfo
{
    public string RoomId { get; init; } = "";
    public string RoomName { get; init; } = "";
    public string MeterId { get; init; } = "";
}

public record ReserveInfo
{
    public decimal Balance { get; init; }
    public decimal CumulativeUsage { get; init; }
    public string ReadTime { get; init; } = "";
}

public record DayUsageItem
{
    public string Date { get; init; } = "";
    public decimal Usage { get; init; }
    public decimal Cost { get; init; }
    public string StartReading { get; init; } = "";
    public string EndReading { get; init; } = "";
}
