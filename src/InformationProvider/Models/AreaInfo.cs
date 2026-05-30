namespace InformationProvider.Models;

public record AreaSummary
{
    public string AreaId { get; init; } = "";
    public string AreaName { get; init; } = "";
}

public record ArchitectureSummary
{
    public string ArchitectureId { get; init; } = "";
    public string ArchitectureName { get; init; } = "";
    public int StoryCount { get; init; }
}

public record RoomSummary
{
    public string RoomNo { get; init; } = "";
    public string RoomName { get; init; } = "";
}
