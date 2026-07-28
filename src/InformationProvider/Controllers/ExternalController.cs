using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using InformationProvider.Data;
using InformationProvider.Models;
using InformationProvider.Services;

namespace InformationProvider.Controllers;

[ApiController]
[Route("api/external")]
public class ExternalController : ControllerBase
{
    private readonly IWhuApiService _whuApi;
    private readonly ElectricityDbContext _db;

    public ExternalController(IWhuApiService whuApi, ElectricityDbContext db)
    {
        _whuApi = whuApi;
        _db = db;
    }

    [HttpGet("daily-usage")]
    public async Task<IActionResult> GetDailyUsage(
        [FromQuery] string roomId,
        [FromQuery] DateTime date,
        CancellationToken ct)
    {
        var today = DateTime.Today;
        var requestDate = date.Date;

        if (requestDate > today)
            return Ok(new { error = "日期无效!" });

        var roomInDb = await _db.DailyUsageRecords
            .AnyAsync(r => r.RoomId == roomId, ct);

        if (!roomInDb)
        {
            try
            {
                await _whuApi.GetBalanceAsync(roomId, ct);
            }
            catch (InvalidOperationException)
            {
                return Ok(new { error = "无该房间!" });
            }
            catch (HttpRequestException)
            {
                return Ok(new { error = "API 请求过于频繁，请稍后重试" });
            }
        }

        var cursor = today.AddDays(-1);  // 始终从昨天开始往前回填

        while (cursor >= DateTime.MinValue)
        {
            var cursorStr = $"{cursor.Year}-{cursor.Month}-{cursor.Day}";

            var exists = await _db.DailyUsageRecords
                .AnyAsync(r => r.RoomId == roomId && r.Date == cursorStr, ct);
            if (exists)
                break;

            DailyUsageResponse daily;
            try
            {
                daily = await _whuApi.GetDailyUsageAsync(roomId, cursor, ct);
            }
            catch
            {
                break;
            }

            if (string.IsNullOrEmpty(daily.StartReading) || string.IsNullOrEmpty(daily.EndReading))
                break;

            _db.DailyUsageRecords.Add(new DailyUsageRecord
            {
                RoomId = roomId,
                Date = cursorStr,
                Usage = daily.Usage.Amount,
                Cost = daily.Cost.Amount
            });

            cursor = cursor.AddDays(-1);

            try
            {
                await Task.Delay(500, ct);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }

        await _db.SaveChangesAsync(ct);

        if (requestDate == today)
        {
            DailyUsageResponse daily;
            try
            {
                daily = await _whuApi.GetDailyUsageAsync(roomId, requestDate, ct);
            }
            catch
            {
                return Ok(new { error = "暂无该表具的日用量信息" });
            }

            return Ok(new
            {
                roomId,
                date = $"{requestDate.Year}-{requestDate.Month}-{requestDate.Day}",
                usage = daily.Usage.Amount,
                cost = daily.Cost.Amount
            });
        }

        var recordDate = $"{requestDate.Year}-{requestDate.Month}-{requestDate.Day}";
        var record = await _db.DailyUsageRecords
            .FirstOrDefaultAsync(r => r.RoomId == roomId && r.Date == recordDate, ct);

        if (record is null)
            return Ok(new { error = "暂无该表具的日用量信息" });

        return Ok(new
        {
            roomId = record.RoomId,
            date = record.Date,
            usage = record.Usage,
            cost = record.Cost
        });
    }
}
