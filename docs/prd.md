# PRD: InformationProvider — 校园电费信息查询系统

## Problem Statement

武汉大学学生目前只能通过登录校园供电系统 Web 页面（zwhqbsd.whu.edu.cn）查询宿舍电费余额和日用电量。这个过程不够便捷——学生需要打开浏览器、输入学号密码、跳转到对应的建筑和楼层才能找到自己的房间。

学生希望能在 QQ 上直接通过房间号查询电费余额、日用电量，并查看一段时间内的余额变化趋势。学校 API 有未公开的频率限制，且每次查询都需要完整的 SM2 加密认证流程，不适合在用户每次查询时都调用。

## Solution

构建一个中间层服务（InformationProvider），对接武汉大学供电系统 API（WhuApi），将数据缓存在本地 SQLite 数据库中作为唯一数据源。用户通过 QQ 机器人与系统交互，QQ 机器人调用 InformationProvider 的 HTTP API 获取数据。在树莓派上以原生 Rust 二进制和 systemd 部署。

### 核心机制

- **订阅制跟踪**：用户通过 QQ 机器人提交 RoomId 订阅房间。订阅后，系统在后台从 WhuApi 回填该房间所有历史日用电量数据到本地数据库，并提供任务状态查询。
- **每日定时任务**：每天凌晨，系统遍历所有已订阅房间，从 WhuApi 拉取前一天的用电量/金额和当前余额，写入数据库。
- **数据库作为唯一数据源**：所有用户查询只读数据库，不调用 WhuApi。WhuApi 仅在订阅回填和定时任务中被调用。
- **手动刷新（Refresh）**：用户可主动触发某个房间的实时数据同步，有独立的严格限频规则。

## User Stories

1. As a 学生, I want to 通过房间号查询当前电费余额, so that 我可以在线了解是否需要充值
2. As a 学生, I want to 通过房间号查询某日的用电度数, so that 我可以了解宿舍用电情况
3. As a 学生, I want to 通过房间号查询某日的用电金额, so that 我可以了解电费消耗
4. As a 学生, I want to 查看一段时间内（如近 7 天/30 天）的用电量趋势图, so that 我可以发现用电高峰和异常
5. As a 学生, I want to 查看一段时间内的电费金额趋势图, so that 我可以了解电费消耗速度
6. As a 学生, I want to 查看一段时间内的余额下降趋势图, so that 我可以预估何时需要充值
7. As a 学生, I want to 订阅一个房间（输入 RoomId）, so that 该房间的数据会被持续追踪
8. As a 学生, I want to 订阅后立即获得当前余额，并在历史回填完成后查询完整用量, so that 我不必等待回填任务才知道余额
9. As a 学生, I want to 在感觉数据异常时主动触发一次刷新, so that 我可以获取最新数据
10. As a QQ bot 维护者, I want to 调用订阅 API（传入 RoomId）, so that 用户可以订阅房间
11. As a QQ bot 维护者, I want to 调用余额查询 API, so that 我可以展示余额给用户
12. As a QQ bot 维护者, I want to 调用日用量查询 API, so that 我可以展示每日用电信息
13. As a QQ bot 维护者, I want to 调用趋势区间查询 API, so that 我可以绘制三合一趋势图
14. As a QQ bot 维护者, I want to 调用刷新 API, so that 用户可以手动触发数据更新
15. As a 系统管理员, I want to 配置查询限频间隔, so that 我可以控制对 WhuApi 的调用压力
16. As a 系统管理员, I want to 配置刷新限频间隔或关闭刷新功能, so that 我可以防止滥用
17. As a 系统管理员, I want to 查看定时任务的运行状态, so that 我可以确认数据是否正常更新
18. As a 系统管理员, I want to 通过 systemd 管理树莓派上的服务, so that 服务可以开机启动并自动重启

## Implementation Decisions

### 架构

- **语言/框架**：Rust / Axum + Tokio
- **数据库**：SQLite（文件数据库，零运维成本，适合单机部署）
- **数据访问 / migration**：sqlx + sqlx migrations
- **部署**：树莓派原生二进制，由 systemd 管理

### 模块划分

- **HTTP handlers**：提供地理信息、余额、日用量、趋势、订阅、回填状态、刷新和健康检查端点
- **WHU integration**：复用 Rust 版 `WhuApiService` / `RoomService`，通过 SM2 + JWT 访问上游
- **Background tasks**：后台异步回填；Asia/Shanghai 每日 02:00 增量更新
- **Persistence**：sqlx `Store`，维护订阅、日用量、余额和回填任务表
- **Rate limiting**：按房间与查询类型记录进程内限频状态

### 数据模型

**Subscription**
| 字段 | 类型 | 说明 |
|------|------|------|
| RoomId | string (PK) | 房间号 |
| RoomName | string | 房间名称（冗余，方便展示） |
| MeterId | string | WHU 电表号 |
| SubscribedAt | DateTime | 订阅时间 |
| MeterId | string | WHU 电表号 |

**DailyUsageRecord**
| 字段 | 类型 | 说明 |
|------|------|------|
| Id | int (PK) | 自增 |
| RoomId | string | 房间号 |
| Date | string (yyyy-MM-dd) | 日期 |
| Usage | REAL | 日用电量（度） |
| Cost | REAL | 日电费（元） |

唯一索引：`(RoomId, Date)`

**DailyBalanceRecord**
| 字段 | 类型 | 说明 |
|------|------|------|
| Id | int (PK) | 自增 |
| RoomId | string | 房间号 |
| Date | string (yyyy-MM-dd) | 日期 |
| Balance | REAL | 余额（元） |

唯一索引：`(RoomId, Date)`

### API 端点

| 方法 | 路径 | 说明 | 限频 |
|------|------|------|------|
| GET | `/api/rooms/{roomId}/balance` | 查询当前余额 | 1次/1h/房间 |
| GET | `/api/rooms/{roomId}/daily-usage?date=yyyy-MM-dd` | 查询某日用量 | 1次/1h/房间 |
| GET | `/api/rooms/{roomId}/trend?startDate=yyyy-MM-dd&endDate=yyyy-MM-dd` | 区间趋势（用电量+金额+余额） | 1次/1h/房间 |
| POST | `/api/rooms` | 订阅房间（body: `{roomId}`） | 无限制 |
| GET | `/api/rooms/{roomId}/backfill` | 查询订阅历史回填状态 | 无限制 |
| POST | `/api/rooms/{roomId}/refresh` | 手动刷新数据 | 1次/30min/房间，可关闭 |

### 回填策略

- 订阅时：异步从昨天开始按 100 天区间向前调用 `GetMeterDayValue`，直到 API 返回空
- 定时任务：每天凌晨拉取前一天的 `GetMeterDayValue` + `GetReserve`
- 余额：不回溯，仅从订阅之日起每日记录
- 刷新：触发对 `GetReserve` + 当天 `GetMeterDayValue` 的调用，写入 DB

### 定时任务

- 由 Tokio 后台任务按 Asia/Shanghai 时区运行
- 运行时间：每天凌晨 2:00（可配置）
- 遍历所有 `Subscription` 表中的房间
- 对每个房间：
  1. 调用 `GetMeterDayValue` 获取前一天数据 → 写入 `DailyUsageRecord`
  2. 调用 `GetReserve` 获取当前余额 → 写入 `DailyBalanceRecord`
- 单个房间失败后每 15 分钟固定间隔重试，直到成功

### 速率限制

- 查询限频：1 小时内同一房间 + 同类型查询最多 1 次
- 刷新限频：30 分钟内同一房间最多 1 次
- 从环境变量读取配置，非硬编码

## Testing Decisions

### 测试原则

- 只测外部行为，不测实现细节
- 使用 SQLite 内存数据库做数据库测试
- 通过 WHU 查询接口 seam 注入 fake，隔离上游网络调用

### Seam 1: Controller (HTTP API)

测试场景：
- 订阅房间 → 验证持久化任务已创建并返回 `202 Accepted`
- 回填状态查询 → 验证状态转换和失败信息可见
- 查询余额（DB 有数据）→ 验证返回正确的余额
- 查询余额（DB 无数据）→ 验证返回 404 / 空数据
- 查询趋势区间 → 验证返回三个数组（用电量、金额、余额）
- 刷新房间 → 验证调用了 WhuApi 并写入了 DB
- 超出限频 → 验证返回 429 Too Many Requests
- 重复订阅同一房间 → 验证幂等处理

### Seam 2: ScheduledTask

测试场景：
- 所有房间正常 → 验证每个房间的数据被正确写入 DB
- 单个房间 WhuApi 失败 → 验证该房间跳过并记录日志，其他房间继续
- 失败后重试 → 验证重试逻辑在 15 分钟后执行

### 不测试的场景

- WHU API 本身的正确性（由集成测试覆盖）
- 地理信息查询（Area → Building → Room）
- SM2 加密和 Token 认证细节

## Out of Scope

- QQ 机器人本身（独立项目）
- Web UI 前端（无需求）
- 用户认证系统（API 预留扩展性但初期不实现）
- 多级告警系统（仅基础日志）
- 历史余额数据迁移（仅从订阅日起采集）
- 多校区数据隔离（所有数据在同一个 DB 中）
- 数据归档和清理策略（未定，未来补充）

## Further Notes

- 数据库（`electricity.db`）应通过 `.gitignore` 排除，不提交到版本控制
- 账号密码和连接配置通过环境变量提供；不得把生产凭据提交到版本控制
- QQ 机器人通过 RoomId 查询，不需要地理信息（Area/Building/Floor）—— 地理信息仅供首次订阅流程使用
