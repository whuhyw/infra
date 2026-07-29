# InformationProvider

校园电费信息查询系统。对接武汉大学（WHU）供电系统 API，为学生提供宿舍电费余额、用电量查询服务。

## Language

### 核心实体

**Room**:
一个学生宿舍房间，由 WHU 系统分配唯一 `RoomId`（纯数字字符串，可能有前导零），有人可读的 `RoomName`（如"枫园14舍-316"）。
_Avoid_: 寝室, 宿舍

**Meter**:
绑定到房间的智能电表，有唯一 `MeterId`。一个房间有且仅有一个电表。
_Avoid_: 表具, 电度表

### 用量数据

**DailyElectricityUsage**:
房间某一天的用电度数。单位：度（kWh）。
_Avoid_: 日用电, 用电量

**DailyElectricityCost**:
房间某一天对应的电费金额。单位：元。
_Avoid_: 日电费, 费用

**MeterReading**:
电表的抄表数值。包含起度（StartReading，本周期开始时读数）和止度（EndReading，本周期结束时读数）。日用电量 = 止度 − 起度。
_Avoid_: 读数, 表底

**CumulativeUsage**:
房间电表安装以来累计用电总量。单位：度。
_Avoid_: 总用量, 累计用电

**Balance**:
房间电费账户余额。单位：元。
_Avoid_: 剩余金额, 余额金额

### 地理层级

**Area**:
校园地理区域，如"文理学部"、"工学部"、"信息学部"等。
_Avoid_: 校区, 片区

**Building**:
一栋宿舍楼，属于某个 Area。有楼层数（StoryCount）。
_Avoid_: 楼栋, 宿舍楼

**Floor**:
Building 中的楼层编号。

### 系统概念

**WhuApi**:
武汉大学供电系统的 HTTP API。需要先通过 SM2 加密认证获取 JWT token，后续请求携带 token。
_Avoid_: 学校 API, 供电 API, WHU 接口

**InformationProviderApi**:
本系统对外提供的 HTTP API，供 QqBot 调用。包括余额查询、日用量查询、趋势图区间查询、房间订阅等端点。
_Avoid_: 后端 API, 后台接口

**DailyUsageRecord**:
本地 SQLite 数据库中缓存的日用量记录。存储字段：`RoomId`, `Date`, `Usage`（日用电量度数）, `Cost`（日用电金额）。是 WhuApi 返回数据的本地副本。

**DailyBalanceRecord**:
本地 SQLite 数据库中缓存的每日余额记录。存储字段：`RoomId`, `Date`, `Balance`（余额金额）。用于生成余额下降折线图。

**Subscription**:
用户将某个 Room 注册到本系统，使其进入定时任务的跟踪范围。订阅后，该 Room 的数据会被每日自动更新。
_Avoid_: 关注, 绑定

**ScheduledTask**:
每天凌晨运行的定时作业。遍历所有已订阅的 Room，从 WhuApi 拉取前一天的用电量/金额和当前余额，写入 DailyUsageRecord 和 DailyBalanceRecord。单个 Room 失败时每 15 分钟固定间隔重试。
_Avoid_: 定时任务, 后台作业

**QqBot**:
本系统的用户界面。用户通过 QQ 与机器人交互，输入房间号查询电费余额、日用量，或查看余额趋势图。
_Avoid_: 聊天机器人, QQ 机器人

**Backfill**:
将指定 Room 的历史日用电量数据从 WhuApi 拉取并写入 DailyUsageRecord 的过程。在订阅时一次性回填所有历史数据（从昨天开始按 100 天分块批量向前查询，直到 API 返回空）。DailyBalanceRecord 无法回填（WhuApi 不提供历史余额接口）。

**DatabaseAsSourceOfTruth**:
数据库是本系统唯一的数据来源。所有用户查询（余额、日用量、趋势图）只读数据库，不调用 WhuApi。WhuApi 仅在订阅回填和定时任务中被调用。查询时若数据库中查不到所需数据，视为数据不存在，而非触发 API 调用。

**RateLimit**:
用户通过 InformationProviderApi 查询实时数据的频率限制。同房间同类型查询 1 小时内最多 1 次，非硬编码（可配置，未来可能按用户粒度）。
_Avoid_: 限频, 节流

**Refresh**:
用户通过 `POST /api/refresh?roomId=xxx` 手动触发某个房间的数据更新（向 WhuApi 发起实时调用并同步数据到 DB）。比普通查询更严格的限频（默认为 30 分钟内最多 1 次），可在配置中单独关闭。
_Avoid_: 刷新, 手动同步
