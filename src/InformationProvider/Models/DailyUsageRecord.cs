using System.ComponentModel.DataAnnotations;

namespace InformationProvider.Models;

public class DailyUsageRecord
{
    [Key]
    public int Id { get; init; }

    public string RoomId { get; init; } = "";

    public string Date { get; init; } = "";

    public decimal Usage { get; init; }

    public decimal Cost { get; init; }
}
