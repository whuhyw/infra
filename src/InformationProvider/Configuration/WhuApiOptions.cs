namespace InformationProvider.Configuration;

public class WhuApiOptions
{
    public const string SectionName = "WhuApi";

    public string BaseUrl { get; set; } = "http://zwhqbsd.whu.edu.cn/ICBS_V2_Server";
    public string AccountId { get; set; } = "";
    public string AccountPass { get; set; } = "";
    public string SysId { get; set; } = "1";
    public string Sm2PublicKey { get; set; } = "04b238b7d42c87a25e1a4eaddca81e8f33fd95773ccd471408e4195db62aa4085f6e0a3b695cad8d60acece1af348f534b7f72d312f2966b144248c9c590c930fc";
}
