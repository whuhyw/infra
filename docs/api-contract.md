# API Contract

**Base URL**: 由部署配置决定（默认 `http://localhost:5051`）
**Content-Type**: `application/json`

---

## 查询校区列表

```
GET /api/areas
```

用于 QQ 机器人交互式查找房间的第一步——用户选择校区。

响应 `200 OK`：
```json
[
    { "areaId": "0025", "areaName": "信息学部学生宿舍" },
    { "areaId": "0001", "areaName": "文理学部学生宿舍" },
    { "areaId": "0017", "areaName": "工学部学生宿舍" },
    { "areaId": "0023", "areaName": "医学部学生宿舍" }
]
```

---

## 查询宿舍楼列表

```
GET /api/areas/{areaId}/buildings
```

用于 QQ 机器人交互式查找房间的第二步——用户选择宿舍楼。

响应 `200 OK`：
```json
[
    { "architectureId": "000267", "architectureName": "信息学部19舍(西区C3)", "storyCount": 7 },
    { "architectureId": "000268", "architectureName": "信息学部20舍(西区C4)", "storyCount": 7 }
]
```

---

## 查询房间列表

```
GET /api/buildings/{buildingId}/rooms?floor={n}
```

用于 QQ 机器人交互式查找房间的第三步——用户选择楼层后展示房间列表。

参数：
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| floor | int | 是 | 楼层号（1 开始） |

响应 `200 OK`：
```json
[
    { "roomNo": "50081020", "roomName": "601" },
    { "roomNo": "50081021", "roomName": "602" },
    { "roomNo": "50081022", "roomName": "603" }
]
```

---

## 订阅房间

```
POST /api/rooms
```

请求体：
```json
{
    "roomId": "123456"
}
```

响应 `200 OK`：
```json
{
    "roomId": "123456",
    "roomName": "枫园14舍-316",
    "subscribedAt": "2026-07-29T10:00:00Z"
}
```

响应 `409 Conflict`（已订阅）：
```json
{
    "error": "该房间已订阅"
}
```

**说明**：
- 触发回填：从昨日开始按 100 天分块批量回溯 `GetMeterDayValue`，直到 API 返回空
- 回填完成后自动调用一次 `GetReserve` 记录当前余额
- 回填期间接口保持同步等待（通常 1-3 秒）

---

## 查询余额

```
GET /api/rooms/{roomId}/balance
```

响应 `200 OK`：
```json
{
    "roomId": "123456",
    "balance": 50.75,
    "date": "2026-07-29"
}
```

响应 `404 Not Found`（无数据）：
```json
{
    "error": "暂无该房间的余额数据"
}
```

**说明**：
- 只读数据库，不调 WhuApi
- 返回最近一条余额记录
- 限频：1 次/小时/房间（可配置）

---

## 查询日用量

```
GET /api/rooms/{roomId}/daily-usage?date=2026-07-28
```

参数：
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| date | string | 是 | 日期，格式 `yyyy-MM-dd`。不能查询未来日期。 |

响应 `200 OK`：
```json
{
    "roomId": "123456",
    "date": "2026-07-28",
    "usage": 12.5,
    "cost": 6.75
}
```

响应 `404 Not Found`：
```json
{
    "error": "暂无该日期的用量数据"
}
```

**说明**：
- 只读数据库，不调 WhuApi
- 限频：1 次/小时/房间（可配置）

---

## 查询趋势区间

```
GET /api/rooms/{roomId}/trend?startDate=2026-07-01&endDate=2026-07-28
```

参数：
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| startDate | string | 是 | 开始日期，格式 `yyyy-MM-dd` |
| endDate | string | 是 | 结束日期，格式 `yyyy-MM-dd` |

响应 `200 OK`：
```json
{
    "roomId": "123456",
    "startDate": "2026-07-01",
    "endDate": "2026-07-28",
    "usage": [
        { "date": "2026-07-01", "value": 10.2 },
        { "date": "2026-07-02", "value": 8.5 }
    ],
    "cost": [
        { "date": "2026-07-01", "value": 5.51 },
        { "date": "2026-07-02", "value": 4.59 }
    ],
    "balance": [
        { "date": "2026-07-01", "value": 100.0 },
        { "date": "2026-07-02", "value": 95.0 }
    ]
}
```

**说明**：
- 三个数组始终同时返回，即使某个数组为空
- 缺失日期的值：跳过的日期不出现在对应数组中
- 只读数据库，不调 WhuApi
- 限频：1 次/小时/房间（可配置）

---

## 手动刷新

```
POST /api/rooms/{roomId}/refresh
```

响应 `200 OK`：
```json
{
    "roomId": "123456",
    "balance": 48.5,
    "usage": {
        "date": "2026-07-29",
        "usage": 11.3,
        "cost": 6.10
    }
}
```

响应 `429 Too Many Requests`：
```json
{
    "error": "刷新过于频繁，请稍后重试"
}
```

响应 `503 Service Unavailable`（刷新功能已关闭）：
```json
{
    "error": "刷新功能已关闭"
}
```

**说明**：
- 调用 `GetReserve` + 当天 `GetMeterDayValue` 并写入 DB
- 限频：1 次/30 分钟/房间
- 可在配置中完全关闭刷新功能

---

## 错误响应格式

所有端点使用一致的错误格式：

```json
{
    "error": "描述错误原因"
}
```

HTTP 状态码：
| 状态码 | 含义 |
|--------|------|
| 200 | 成功 |
| 400 | 请求参数错误（日期格式错误等） |
| 404 | 数据不存在 |
| 409 | 资源冲突（如重复订阅） |
| 429 | 超过频率限制 |
| 503 | 功能不可用（如刷新功能已关闭） |
