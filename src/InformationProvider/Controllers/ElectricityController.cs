using Microsoft.AspNetCore.Mvc;
using InformationProvider.Services;
using InformationProvider.Models;

namespace InformationProvider.Controllers;

[ApiController]
[Route("api/electricity")]
public class ElectricityController : ControllerBase
{
    private readonly IWhuApiService _whuApi;
    private readonly IRoomService _roomService;

    public ElectricityController(IWhuApiService whuApi, IRoomService roomService)
    {
        _whuApi = whuApi;
        _roomService = roomService;
    }

    [HttpGet("balance")]
    public async Task<ActionResult<ElectricityBalanceResponse>> GetBalance(
        [FromQuery] string roomId,
        CancellationToken ct)
    {
        try
        {
            var result = await _whuApi.GetBalanceAsync(roomId, ct);
            return Ok(result);
        }
        catch (InvalidOperationException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }

    [HttpGet("daily-usage")]
    public async Task<ActionResult<DailyUsageResponse>> GetDailyUsage(
        [FromQuery] string roomId,
        [FromQuery] DateTime? date,
        CancellationToken ct)
    {
        try
        {
            var queryDate = date ?? DateTime.Now.AddDays(-1);
            var result = await _whuApi.GetDailyUsageAsync(roomId, queryDate, ct);
            return Ok(result);
        }
        catch (InvalidOperationException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }

    [HttpGet("areas")]
    public async Task<ActionResult<List<AreaSummary>>> GetAreas(CancellationToken ct)
    {
        var areas = await _roomService.GetAreasAsync(ct);
        return Ok(areas);
    }

    [HttpGet("areas/{areaId}/buildings")]
    public async Task<ActionResult<List<ArchitectureSummary>>> GetBuildings(
        string areaId,
        CancellationToken ct)
    {
        var buildings = await _roomService.GetArchitecturesAsync(areaId, ct);
        return Ok(buildings);
    }

    [HttpGet("buildings/{buildingId}/rooms")]
    public async Task<ActionResult<List<RoomSummary>>> GetRooms(
        string buildingId,
        [FromQuery] int floor,
        CancellationToken ct)
    {
        var rooms = await _roomService.GetRoomsAsync(buildingId, floor, ct);
        return Ok(rooms);
    }
}
