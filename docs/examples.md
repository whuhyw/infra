# API 使用示例

以下示例假设服务运行在 `http://localhost:5051`。

---

## 订阅房间

```bash
curl -X POST http://localhost:5051/api/rooms \
  -H "Content-Type: application/json" \
  -d '{"roomId": "314"}'
```

响应：
```json
{
    "roomId": "314",
    "roomName": "枫园14舍-316",
    "subscribedAt": "2026-07-29T10:00:00Z"
}
```

重复订阅：
```bash
curl -X POST http://localhost:5051/api/rooms \
  -H "Content-Type: application/json" \
  -d '{"roomId": "314"}'
```

响应：
```json
{
    "error": "该房间已订阅"
}
```

---

## 查询余额

```bash
curl http://localhost:5051/api/rooms/314/balance
```

响应：
```json
{
    "roomId": "314",
    "balance": 50.75,
    "date": "2026-07-29"
}
```

---

## 查询日用电量

```bash
curl "http://localhost:5051/api/rooms/314/daily-usage?date=2026-07-28"
```

响应：
```json
{
    "roomId": "314",
    "date": "2026-07-28",
    "usage": 12.5,
    "cost": 6.75
}
```

---

## 查询趋势区间

```bash
curl "http://localhost:5051/api/rooms/314/trend?startDate=2026-07-01&endDate=2026-07-28"
```

响应：
```json
{
    "roomId": "314",
    "startDate": "2026-07-01",
    "endDate": "2026-07-28",
    "usage": [
        { "date": "2026-07-01", "value": 10.2 },
        { "date": "2026-07-02", "value": 8.5 },
        { "date": "2026-07-03", "value": 12.1 }
    ],
    "cost": [
        { "date": "2026-07-01", "value": 5.51 },
        { "date": "2026-07-02", "value": 4.59 },
        { "date": "2026-07-03", "value": 6.53 }
    ],
    "balance": [
        { "date": "2026-07-01", "value": 100.0 },
        { "date": "2026-07-02", "value": 94.41 },
        { "date": "2026-07-03", "value": 89.82 }
    ]
}
```

---

## 手动刷新数据

```bash
curl -X POST http://localhost:5051/api/rooms/314/refresh
```

响应：
```json
{
    "roomId": "314",
    "balance": 48.5,
    "usage": {
        "date": "2026-07-29",
        "usage": 11.3,
        "cost": 6.10
    }
}
```

超限频：
```bash
curl -X POST http://localhost:5051/api/rooms/314/refresh
```

响应：
```json
{
    "error": "刷新过于频繁，请稍后重试"
}
```

---

## 查询不存在的数据

```bash
curl http://localhost:5051/api/rooms/999/balance
```

响应：
```json
{
    "error": "暂无该房间的余额数据"
}
```
