# Operations

## 部署

### Docker

```dockerfile
FROM mcr.microsoft.com/dotnet/aspnet:10.0 AS base
WORKDIR /app
EXPOSE 5051

FROM mcr.microsoft.com/dotnet/sdk:10.0 AS build
WORKDIR /src
COPY ["src/InformationProvider/InformationProvider.csproj", "InformationProvider/"]
RUN dotnet restore "InformationProvider/InformationProvider.csproj"
COPY src/ .
WORKDIR "/src/InformationProvider"
RUN dotnet publish -c Release -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=build /app/publish .
ENTRYPOINT ["dotnet", "InformationProvider.dll"]
```

### docker-compose

```yaml
version: "3.8"
services:
  information-provider:
    build: .
    ports:
      - "5051:5051"
    volumes:
      - ./data:/app/data  # 持久化 SQLite 文件
    environment:
      - ASPNETCORE_ENVIRONMENT=Production
      - ASPNETCORE_URLS=http://+:5051
      - ConnectionStrings__Default=Data Source=data/electricity.db
    restart: unless-stopped
```

## 配置

所有配置通过 `appsettings.json` + 环境变量覆盖：

| 环境变量 | 说明 | 默认值 |
|----------|------|--------|
| `WhuApi__AccountId` | WHU API 账号 | （必填） |
| `WhuApi__AccountPass` | WHU API 密码 | （必填） |
| `ConnectionStrings__Default` | SQLite 连接串 | `Data Source=electricity.db` |
| `RateLimiting__QueryInterval` | 查询限频间隔 | `01:00:00` |
| `RateLimiting__RefreshInterval` | 刷新限频间隔 | `00:30:00` |
| `RateLimiting__RefreshEnabled` | 是否启用刷新 | `true` |
| `ScheduledTask__RunTime` | 定时任务执行时间 | `02:00` |
| `ScheduledTask__RetryInterval` | 失败重试间隔 | `00:15:00` |
| `ScheduledTask__MaxConcurrent` | 最大并发房间数 | `5` |

## 数据持久化

- SQLite 文件默认生成在容器工作目录
- 建议挂载 volume 到 `./data/` 目录
- `.gitignore` 应排除 `*.db` 和 `data/` 目录

## 监控

### 日志

- 使用 ASP.NET Core 内置 `ILogger`
- 结构化日志输出到 stdout
- 关键日志事件：
  - 定时任务开始/结束（含处理房间数）
  - 单个房间更新成功/失败
  - 限频触发警告
  - WhuApi 调用异常

### 健康检查

```
GET /health
```

响应：
```json
{
    "status": "healthy",
    "dbConnected": true,
    "lastScheduledRun": "2026-07-29T02:00:00Z",
    "subscribedRooms": 42
}
```

## 备份

- 定期拷贝 `electricity.db` 文件即可完成备份
- 备份周期建议：每日
- 使用 `sqlite3 electricity.db .backup backup.db` 做在线备份（SQLite 支持 WAL 模式热备）

## 常见问题

| 问题 | 排查 |
|------|------|
| 学生说"查不到数据" | 检查订阅是否成功？定时任务是否正常运行？ |
| 定时任务失败 | 检查 WhuApi 连通性、账号有效性、日志中的错误信息 |
| 数据库损坏 | 从备份恢复，或重新订阅房间（会重新回填） |
| 余额长时间不更新 | 检查刷新功能是否被关闭？WhuApi 是否限频？ |
