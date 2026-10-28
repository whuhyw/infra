# Implementation Decisions

## ADR 索引

| # | 标题 | 状态 |
|---|------|------|
| 0001 | Database as Source of Truth | accepted |

详见 [docs/adr/0001-database-as-source-of-truth.md](./adr/0001-database-as-source-of-truth.md)。

## 决策概要

### 数据库作为唯一数据源

**决策**：所有用户查询只读 SQLite 缓存数据库，不调用 WHU API。WhuApi 仅在订阅回填和定时任务中被调用。

**理由**：WHU API 有未公开的频率限制。将查询和 API 调用解耦，提供稳定的查询体验。

### 固定间隔重试（非指数退避）

**决策**：定时任务单个房间失败后每 15 分钟固定间隔重试。

**理由**：失败诱因固定（限频/临时网络问题），单任务无惊群风险，固定间隔更简单。

### 订阅即回填

**决策**：订阅时一次性回溯所有历史日用电量数据。

**理由**：用户订阅后应能查询历史数据。定时任务只负责增量更新。

### 余额不可回溯

**决策**：DailyBalanceRecord 从订阅之日起采集，无法补历史。

**理由**：WhuApi 的 `GetReserve` 只返回当前余额，不提供历史接口。

### 三合一趋势图

**决策**：用电量、电费、余额通过同一个区间查询端点返回。

**理由**：前端（QQ bot）通常需要同时绘制三张趋势图，单次返回减少网络开销。

### Refresh 端点设计

**决策**：提供明确的 `POST /api/rooms/{roomId}/refresh` 端点，比查询更严限频，支持配置关闭。

**理由**：覆盖定时任务空窗期的数据延迟，同时防止滥用导致触达 WHU API 限频。

### Rust 实现栈

**决策**：HTTP 使用 Axum/Tokio，SQLite 访问和 migration 使用 sqlx；每日任务在 Asia/Shanghai 时区由本进程调度。

**理由**：保持单实例、单 SQLite 文件的部署模型，不增加外部任务调度、数据库或容器运行时依赖。

### 树莓派部署

**决策**：使用匹配树莓派操作系统架构的原生 Rust 二进制，并由 systemd 管理；Docker 不作为部署依赖。

**理由**：目标环境是单台树莓派，systemd 已提供开机启动、重启策略和日志管理。需要在树莓派本机编译或交叉编译到对应 ARM target。

### 异步订阅回填

**决策**：`POST /api/rooms` 返回 `202 Accepted`，通过 `GET /api/rooms/{roomId}/backfill` 查询持久化任务状态。

**理由**：历史回填可能持续数分钟；同步占用 HTTP 请求会造成超时。持久化任务状态允许服务重启后恢复未完成任务。
