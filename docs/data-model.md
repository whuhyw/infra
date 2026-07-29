# Data Model

## 实体关系

```
┌──────────────┐       ┌───────────────────┐
│  Subscription│       │ DailyUsageRecord  │
├──────────────┤       ├───────────────────┤
│ PK RoomId    │       │ PK Id             │
│    RoomName  │       │ FK RoomId         │
│    SubAt     │       │    Date (yyyy-MM-dd)
└──────┬───────┘       │    Usage (度)     │
       │               │    Cost (元)      │
       │               └─────────┬─────────┘
       │                         │
       │               ┌─────────▼─────────┐
       │               │DailyBalanceRecord │
       │               ├───────────────────┤
       └──────────────►│ PK Id             │
                       │ FK RoomId         │
                       │    Date (yyyy-MM-dd)
                       │    Balance (元)   │
                       └───────────────────┘
```

## 表结构

### Subscription

追踪需要定时更新的房间。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| RoomId | TEXT | PK | 房间号，纯数字字符串（可能有前导零） |
| RoomName | TEXT | NOT NULL | 房间名称，如"枫园14舍-316" |
| SubscribedAt | TEXT | NOT NULL | ISO 8601 订阅时间戳 |

**用途**: 定时任务遍历此表确定要更新的房间列表。

### DailyUsageRecord

缓存的每日用电量数据。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| Id | INTEGER | PK AUTOINCREMENT | 自增主键 |
| RoomId | TEXT | NOT NULL, FK → Subscription | 房间号 |
| Date | TEXT | NOT NULL | 日期，格式 `yyyy-MM-dd` |
| Usage | REAL | NOT NULL | 日用电量，单位：度（kWh） |
| Cost | REAL | NOT NULL | 日电费，单位：元 |

**唯一索引**: `(RoomId, Date)`
**用途**: 供日用量查询和趋势图中的用电量/金额曲线使用。

### DailyBalanceRecord

缓存的每日余额快照。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| Id | INTEGER | PK AUTOINCREMENT | 自增主键 |
| RoomId | TEXT | NOT NULL, FK → Subscription | 房间号 |
| Date | TEXT | NOT NULL | 日期，格式 `yyyy-MM-dd` |
| Balance | REAL | NOT NULL | 余额，单位：元 |

**唯一索引**: `(RoomId, Date)`
**用途**: 供余额趋势图使用。

## 数据流

```
WhuApi
  │
  ├── 订阅回填 ──► DailyUsageRecord (100天分块批量回溯)
  │
  ├── 定时任务 ──► DailyUsageRecord (前一天)
  │           ──► DailyBalanceRecord (当前余额)
  │
  └── 主动刷新 ──► DailyUsageRecord (当天)
              ──► DailyBalanceRecord (当前余额)

用户查询 ──► 只读 DailyUsageRecord / DailyBalanceRecord
```

## 约束

- RoomId 在所有表之间一致，由 Subscription 表保证存在
- 同一房号同一天不会产生两条 DailyUsageRecord
- 余额趋势区间查询的缺失日期 = 无数据（不会补默认值）
