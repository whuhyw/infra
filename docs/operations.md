# Operations

## 部署

### 树莓派 systemd 部署

先在树莓派上安装 Rust 工具链和编译依赖，然后在仓库根目录构建 release 二进制。这样产物会与树莓派当前操作系统和 CPU 架构匹配。

```sh
cargo build --locked --release
sudo useradd --system --home-dir /var/lib/infra-service --create-home infra-service
sudo install -m 0755 target/release/infra-service /usr/local/bin/infra-service
sudo install -m 0644 deploy/infra-service.service /etc/systemd/system/infra-service.service
sudo install -d -o infra-service -g infra-service /var/lib/infra-service
sudo install -m 0600 deploy/infra-service.env.example /etc/infra-service.env
# 环境文件可以保持默认值；需要覆盖 WHU 凭据时再编辑
sudo systemctl daemon-reload
sudo systemctl enable --now infra-service
```

服务运行用户为 `infra-service`，SQLite 数据库固定放在 `/var/lib/infra-service/electricity.db`。查看服务日志：

```sh
sudo journalctl -u infra-service -f
```

## 配置

服务从环境变量读取配置：

| 环境变量 | 说明 | 默认值 |
|----------|------|--------|
| `BIND_ADDRESS` | HTTP 监听地址 | `0.0.0.0:5051` |
| `DATABASE_URL` | sqlx SQLite 连接 URL | `sqlite://electricity.db` |
| `WhuApi__AccountId` | WHU API 登录账号 | `ph` |
| `WhuApi__AccountPass` | WHU API 登录密码 | `phAPI` |
| `RateLimiting__QueryInterval` | 查询限频间隔 | `01:00:00` |
| `RateLimiting__RefreshInterval` | 刷新限频间隔 | `00:30:00` |
| `RateLimiting__RefreshEnabled` | 是否启用刷新 | `true` |
| `ScheduledTask__RunTime` | 定时任务执行时间 | `02:00` |
| `ScheduledTask__RetryInterval` | 失败重试间隔 | `00:15:00` |
| `ScheduledTask__MaxConcurrent` | 最大并发房间数 | `5` |
| `Backfill__ChunkSizeDays` | 每个历史回填请求的日期跨度 | `100` |

## 数据持久化

- SQLite 数据库放在 `/var/lib/infra-service/electricity.db`
- 服务使用 WAL 模式；不要在服务运行时只复制主数据库文件，应使用 SQLite 的在线备份命令：

  ```sh
  sqlite3 /var/lib/infra-service/electricity.db ".backup '/var/lib/infra-service/electricity-backup.db'"
  ```
- 服务启动时自动执行 `migrations/` 中的 sqlx migrations
- SQLite 使用 WAL 模式；当前部署模型要求单实例

## 监控

### 日志

- 使用 `tracing` 结构化日志
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

- 定期使用 SQLite `.backup` 命令备份数据库
- 备份周期建议：每日

## 常见问题

| 问题 | 排查 |
|------|------|
| 学生说"查不到数据" | 检查订阅是否成功？定时任务是否正常运行？ |
| 定时任务失败 | 检查 WhuApi 连通性、账号有效性、日志中的错误信息 |
| 数据库损坏 | 从备份恢复，或重新订阅房间（会重新回填） |
| 余额长时间不更新 | 检查刷新功能是否被关闭？WhuApi 是否限频？ |
