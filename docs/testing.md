# Testing

## 测试策略

### Seam 1: Controller (HTTP API)

通过 `WebApplicationFactory` 或 `TestServer` 集成测试，注入 mock 替换 `IWhuApiService`，使用 SQLite InMemory provider。

```csharp
// 测试夹具
public class InfoProviderApiFactory : WebApplicationFactory<Program>
{
    protected override void ConfigureWebHost(IWebHostBuilder builder)
    {
        builder.ConfigureServices(services =>
        {
            // 替换 WhuApiService 为 mock
            services.AddScoped<IWhuApiService>(_ => mockWhuApi.Object);
            // 使用内存 SQLite
            services.AddDbContext<ElectricityDbContext>(options =>
                options.UseSqlite("Data Source=:memory:"));
        });
    }
}
```

### Seam 2: ScheduledTask

直接实例化后台服务，注入 mock `IWhuApiService` 和真实内存数据库。

---

## 测试用例

### 订阅端点 (POST /api/rooms)

| # | 场景 | 预期 |
|---|------|------|
| 1 | 订阅新房间，`GetMeterDayValue` 返回 30 天历史数据 | 返回 200，DB 中有 30 条 DailyUsageRecord |
| 2 | 重复订阅同一房间 | 返回 409，DB 中仍只有 1 条 Subscription |
| 3 | 订阅时 WHU API 返回"无该房间"错误 | 返回 400 |
| 4 | 订阅后立即查询余额 | 返回订阅时写入的余额 |

### 余额查询 (GET /api/rooms/{roomId}/balance)

| # | 场景 | 预期 |
|---|------|------|
| 1 | DB 中有该房间的余额记录 | 返回 200 + 余额数据 |
| 2 | DB 中没有该房间的任何数据 | 返回 404 |
| 3 | 1 小时内重复查询同一房间 | 第 2 次返回 429 |

### 日用量查询 (GET /api/rooms/{roomId}/daily-usage)

| # | 场景 | 预期 |
|---|------|------|
| 1 | DB 中有该日记录 | 返回 200 + 用量数据 |
| 2 | DB 中无该日记录 | 返回 404 |
| 3 | 查询未来日期 | 返回 400 |
| 4 | 日期格式错误（如 2026/07/28） | 返回 400 |

### 趋势区间查询 (GET /api/rooms/{roomId}/trend)

| # | 场景 | 预期 |
|---|------|------|
| 1 | DB 中有完整区间数据 | 返回 200 + 三个非空数组 |
| 2 | DB 中部分日期缺失 | 缺失日期不在对应数组中 |
| 3 | DB 中完全无数据 | 返回 200 + 三个空数组 |
| 4 | startDate > endDate | 返回 400 |

### 刷新端点 (POST /api/rooms/{roomId}/refresh)

| # | 场景 | 预期 |
|---|------|------|
| 1 | 正常刷新 | 调用 WhuApi 并写入 DB，返回 200 |
| 2 | 30 分钟内重复刷新 | 第 2 次返回 429 |
| 3 | 刷新功能在配置中关闭 | 返回 503 |
| 4 | WhuApi 调用失败 | 返回 502 |

### 定时任务

| # | 场景 | 预期 |
|---|------|------|
| 1 | 3 个已订阅房间，全部正常 | 每个房间都有新的 DailyUsageRecord 和 DailyBalanceRecord |
| 2 | 1 个房间 WhuApi 失败 | 失败房间无新数据，其他 2 个正常写入 |
| 3 | 失败后重试（模拟 15 分钟后） | 原失败房间的数据写入成功 |
| 4 | 前一天数据已存在（重复执行） | 不产生重复记录（UPSERT） |
