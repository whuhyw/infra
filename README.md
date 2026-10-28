# InformationProvider（Rust）

本服务以 Rust 重实现仓库中已经存在的 WHU API 封装，并补齐文档规划的 HTTP API、SQLite 缓存、订阅历史回填、每日更新、手动刷新、限频和健康检查。

## 构建与运行

```sh
cargo build --locked --release
./target/release/information-provider-whu
```

运行时必须设置 WHU 账号环境变量。服务默认监听 `0.0.0.0:5051`，数据库为当前目录下的 `electricity.db`，启动时自动运行 `migrations/` 中的 sqlx migrations。树莓派部署应在树莓派上构建，或交叉编译到对应的 ARM target；不同型号可能使用 32 位或 64 位系统。

## 配置

| 环境变量 | 默认值 | 说明 |
|---|---|---|
| `BIND_ADDRESS` | `0.0.0.0:5051` | HTTP 监听地址 |
| `DATABASE_URL` | `sqlite://electricity.db` | SQLite 连接 URL |
| `WhuApi__AccountId` | `ph` | WHU API 登录账号；可用环境变量覆盖 |
| `WhuApi__AccountPass` | `phAPI` | WHU API 登录密码；可用环境变量覆盖 |
| `WhuApi__BaseUrl` | WHU API 默认地址 | WHU API 基地址 |
| `RateLimiting__QueryInterval` | `01:00:00` | 同房间同查询类型的限频间隔 |
| `RateLimiting__RefreshInterval` | `00:30:00` | 手动刷新限频间隔 |
| `RateLimiting__RefreshEnabled` | `true` | 是否启用手动刷新 |
| `ScheduledTask__RunTime` | `02:00` | Asia/Shanghai 每日更新时间 |
| `ScheduledTask__RetryInterval` | `00:15:00` | 单房间更新失败后的重试间隔 |
| `ScheduledTask__MaxConcurrent` | `5` | 定时更新的最大房间并发数 |
| `Backfill__ChunkSizeDays` | `100` | 历史回填每次查询天数 |

## 异步订阅

`POST /api/rooms` 建立订阅和持久化回填任务后立即返回 `202 Accepted`。响应的 `Location` 指向 `GET /api/rooms/{roomId}/backfill`；任务状态为 `completed` 后，订阅房间的历史用量回填完成。进程重启时，未完成任务会自动恢复。

## 验证

```sh
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

树莓派上的 systemd 部署说明见 [操作指南](docs/operations.md#树莓派-systemd-部署)。
