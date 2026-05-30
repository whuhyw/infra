using InformationProvider.Models;

namespace InformationProvider.Services;

public interface IRoomService
{
    Task<List<AreaSummary>> GetAreasAsync(CancellationToken ct = default);
    Task<List<ArchitectureSummary>> GetArchitecturesAsync(string areaId, CancellationToken ct = default);
    Task<List<RoomSummary>> GetRoomsAsync(string buildingId, int floor, CancellationToken ct = default);
}
