namespace InformationProvider.Models;

public record WhuApiResponse<T>
{
    public int Code { get; init; }
    public string Mess { get; init; } = "";
    public T? Data { get; init; }
}

public record WhuSignInData
{
    public string access_Jwt { get; init; } = "";
    public string refresh_Jwt { get; init; } = "";
    public int Result { get; init; }
    public string Msg { get; init; } = "";
}

public record WhuRoomMeterData
{
    public RoomInfoData? roomInfo { get; init; }
    public List<MeterItem> meterList { get; init; } = [];
    public int Result { get; init; }
    public string Msg { get; init; } = "";
}

public record RoomInfoData
{
    public string RoomNo { get; init; } = "";
    public string RoomName { get; init; } = "";
    public string RoomEntierName { get; init; } = "";
    public string LastQueryDate { get; init; } = "";
}

public record MeterItem
{
    public string meterId { get; init; } = "";
    public string meterType { get; init; } = "";
    public string conType { get; init; } = "";
}

public record WhuReserveData
{
    public string remainPower { get; init; } = "";
    public string remainName { get; init; } = "";
    public string ZVlaue { get; init; } = "";
    public string state { get; init; } = "";
    public string valve { get; init; } = "";
    public string unit { get; init; } = "";
    public string readTime { get; init; } = "";
    public int Result { get; init; }
    public string Msg { get; init; } = "";
}

public record WhuDayValueData
{
    public List<DayValueItem> DayValues { get; init; } = [];
    public int Result { get; init; }
    public string Msg { get; init; } = "";
}

public record DayValueItem
{
    public string dayValue { get; init; } = "";
    public string dayUseMeony { get; init; } = "";
    public string dw { get; init; } = "";
    public string StarZValueZY { get; init; } = "";
    public string EndZValueZY { get; init; } = "";
    public string curDayTime { get; init; } = "";
}

public record WhuAreaData
{
    public List<AreaItem> areaInfoList { get; init; } = [];
}

public record AreaItem
{
    public string AreaID { get; init; } = "";
    public string AreaName { get; init; } = "";
}

public record WhuArchitectureData
{
    public List<ArchitectureItem> architectureInfoList { get; init; } = [];
    public int Result { get; init; }
}

public record ArchitectureItem
{
    public string ArchitectureID { get; init; } = "";
    public string ArchitectureName { get; init; } = "";
    public int ArchitectureStorys { get; init; }
}

public record WhuRoomListData
{
    public List<RoomListItem> roomInfoList { get; init; } = [];
}

public record RoomListItem
{
    public string RoomNo { get; init; } = "";
    public string RoomName { get; init; } = "";
}
