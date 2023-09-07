using System.Text.Json.Serialization;

static class Constants
{
    public const string DataDirectory = "../data";
    public const string DiffDirectory = "../diff";
    public const string ResultCsvPath = "../result.csv";
    public const string ResultJsonPath = "../codes.json";
    public static ReadOnlySpan<byte> CsvHeader => "\uFEFF代码,一级行政区,二级行政区,名称,级别,状态,启用时间,变更（弃用）时间,新代码\n"u8;
}

struct Successor : IComparable<Successor>
{
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
    public bool Optional { get; set; }
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
    public uint Time { get; set; }
    public uint Code { get; set; }

    public int CompareTo(Successor other)
    {
        int cmp = Optional.CompareTo(other.Optional);
        if (cmp == 0)
        {
            cmp = Time.CompareTo(other.Time);
        }
        if (cmp == 0)
        {
            cmp = Code.CompareTo(other.Code);
        }
        return cmp;
    }
}

record Entry(uint Time, string? Name, string? ParentName)
{
    public SortedSet<Successor> Attr = new();
}

class JsonEntry
{
    public uint Code { get; set; }
    public string Name { get; set; } = "";
    public uint Start { get; set; }
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public uint? End { get; set; }
    public List<Successor> Successors = new();
    public List<JsonEntry> Children = new();

    [JsonPropertyName("successors")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<Successor>? SerializedSuccessors
    {
        get => Successors.Count > 0 ? Successors : null;
    }

    [JsonPropertyName("children")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<JsonEntry>? SerializedChildren
    {
        get => Children.Count > 0 ? Children : null;
    }
}

class Area
{
    public List<Entry> Entries;
    public bool Deprecated = false;

    public Area(Entry entry)
    {
        Entries = new List<Entry>() { entry };
    }

    public string? LastNameIntersecting(uint start, uint? end)
    {
        int last = Entries.Count - 1;
        if (end == null)
        {
            return Entries[last].Name;
        }

        for (int i = last; i >= 0; i--)
        {
            Entry cur = Entries[i];
            if (i == last && !Deprecated)
            {
                if (cur.Time < end)
                {
                    return cur.Name;
                }
                continue;
            }
            if (cur.Name == null)
            {
                continue;
            }
            if (Entries[i + 1].Time > start && cur.Time < end)
            {
                return cur.Name;
            }
        }
        return null;
    }
}

enum Level
{
    Province,
    Prefecture,
    County,
}

static class Utils
{
    public static IEnumerable<(int, T)> Indexed<T>(this IEnumerable<T> self)
       => self.Select((item, index) => (index, item));

    public static IEnumerable<(uint, string)> ReadData(string path)
    {
        string fileName = Path.GetFileName(path);

        foreach (var (i, line) in File.ReadLines(path).Indexed())
        {
            if (line.Length < 7)
            {
                throw new InvalidDataException($"{fileName}({i}): line too short");
            }
            var code = uint.Parse(line[..6]);
            if (line[6] != '\t')
            {
                throw new InvalidDataException($"{fileName}({i}): no tab");
            }
            yield return (code, line[7..]);
        }
    }

    public static (string, string)? SplitOnce(this string self, params char[] delims)
    {
        int i = self.IndexOfAny(delims);
        return i >= 0 ? (self[..i], self[(i + 1)..]) : null;
    }

    public static uint ParentCode(uint code)
    {
        if (code % 100 != 0)
        {
            return code / 100 * 100;
        }
        else if (code % 10000 != 0)
        {
            return code / 10000 * 10000;
        }
        else
        {
            return 0;
        }
    }

    public static uint CodeDistance(uint a, uint b)
    {
        if (a / 100 == b / 100)
        {
            return 1;
        }
        else if (a / 10000 == b / 10000)
        {
            return 2;
        }
        else
        {
            return 3;
        }
    }

    public static Level LevelFromCode(uint code)
    {
        if (code % 100 != 0)
        {
            return Level.County;
        }
        else if (code % 10000 != 0)
        {
            return Level.Prefecture;
        }
        else
        {
            return Level.Province;
        }
    }

    public static string Description(this Level self) => self switch
    {
        Level.Province => "省级",
        Level.Prefecture => "地级",
        Level.County => "县级",
        _ => throw new InvalidOperationException("unreachable"),
    };
}
