using System.Collections.Immutable;
using System.Diagnostics;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Unicode;

var stopWatch = new Stopwatch();
stopWatch.Start();

var allDict = new Dictionary<uint, Area>(8192);
var curDict = new Dictionary<uint, string>(4096);

foreach (string path in Directory.GetFiles(Constants.DataDirectory))
{
    string fileStem = Path.GetFileNameWithoutExtension(path);

    var time = uint.Parse(fileStem);

    foreach (var (code, name) in Utils.ReadData(path))
    {
        curDict.Add(code, name);
    }

    foreach (var (code, area) in allDict)
    {
        if (!curDict.ContainsKey(code) && !area.Deprecated)
        {
            area.Entries.Add(new Entry(time, null, null));
            area.Deprecated = true;
        }
    }

    foreach (var (code, name) in curDict)
    {
        curDict.TryGetValue(Utils.ParentCode(code), out string? parentName);
        if (allDict.TryGetValue(code, out Area? area))
        {
            Entry last = area.Entries.Last();
            if (last.Name != name || last.ParentName != parentName)
            {
                area.Entries.Add(new Entry(time, name, parentName));
                area.Deprecated = false;
            }
        }
        else
        {
            allDict[code] = new Area(new Entry(time, name, parentName));
        }
    }

    curDict.Clear();
    Console.WriteLine($"Processed: {fileStem}");
}

InsertDiff(allDict);

using FileStream fsCsv = File.Create(Constants.ResultCsvPath);
fsCsv.Write(Constants.CsvHeader);

JsonEntry root = new();

List<uint> keys = allDict.Keys.ToList();
keys.Sort();

foreach (uint code in keys)
{
    Area area = allDict[code];
    List<Entry> entries = area.Entries;
    int last = entries.Count - (area.Deprecated ? 2 : 1);

    for (int i = last; i >= 0; i--)
    {
        Entry entry = entries[i];
        if (entry.Name == null)
        {
            continue;
        }
        uint? end = i + 1 < entries.Count ? entries[i + 1].Time : null;
        WriteEntry(fsCsv, root, allDict, code, entry.Name, entry.Time, end, i == last, entry.Attr);
    }
}

using FileStream fsJson = File.Create(Constants.ResultJsonPath);

var options = new JsonSerializerOptions
{
    Encoder = JavaScriptEncoder.Create(UnicodeRanges.All),
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
};
JsonSerializer.Serialize(fsJson, root.Children, options);

fsCsv.Close();
fsJson.Close();

stopWatch.Stop();
Console.WriteLine($"Finished: {stopWatch.Elapsed}");

void InsertDiff(Dictionary<uint, Area> dict)
{
    foreach (FwdDiff fd in Diff.ReadFwdDiff())
    {
        if (fd.Code == 0)
        {
            continue;
        }
        Area area = dict[fd.Code];
        Entry entry = area.Entries
            .AsEnumerable().Reverse().First(e => e.Time < fd.Time);
        entry.Attr.UnionWith(from code in fd.Attr
                             select new Successor()
                             {
                                 Time = fd.Time,
                                 Code = code,
                                 Optional = fd.Optional,
                             });
    }

    foreach (var (code, area) in dict)
    {
        for (int i = 0; i < area.Entries.Count - 1; i++)
        {
            Entry entry = area.Entries[i];
            uint nextTime = area.Entries[i + 1].Time;

            if (entry.Attr.Count == 0 || entry.Attr.Last().Time != nextTime)
            {
                entry.Attr.Add(new Successor()
                {
                    Time = nextTime,
                    Code = code,
                    Optional = false,
                });
            }
        }
    }
}

void WriteEntry(
    FileStream fs,
    JsonEntry root,
    Dictionary<uint, Area> dict,
    uint code,
    string name,
    uint start,
    uint? end,
    bool isLast,
    SortedSet<Successor> attr
)
{
    JsonEntry entry = root;
    Level level = Utils.LevelFromCode(code);

    uint provCode = code / 10000 * 10000;
    string provName = dict[provCode].Entries[0].Name!;
    string prefName = "";

    if (level != Level.Province)
    {
        entry = entry.Children.First(e => e.Code == provCode);
        if (level == Level.Prefecture)
        {
            prefName = name;
        }
        else
        {
            uint prefCode = code / 100 * 100;
            string? _prefName = null;
            if (Utils.LevelFromCode(prefCode) == Level.Prefecture
                && dict.TryGetValue(prefCode, out Area? area))
            {
                _prefName = area.LastNameIntersecting(start, end);
            }

            if (_prefName != null)
            {
                entry = entry.Children.First(e => e.Code == prefCode && e.Start <= start);
            }
            prefName = _prefName ?? "直辖";
        }
    }

    entry.Children.Add(new JsonEntry()
    {
        Code = code,
        Name = name,
        Start = start,
        End = end,
        Successors = (from su in attr
                      where !su.Optional
                      select new Successor()
                      {
                          Time = su.Time != end ? su.Time : 0,
                          Code = su.Code,
                          Optional = su.Optional
                      }).ToList(),
    });

    string status;
    if (end == null)
    {
        status = "启用";
    }
    else if (isLast)
    {
        status = "弃用";
    }
    else
    {
        status = "变更";
    }

    fs.Write(Encoding.UTF8.GetBytes($"{code},{provName},{prefName},{name},{level.Description()},{status},{start},"));
    if (end != null)
    {
        fs.Write(Encoding.UTF8.GetBytes($"{end}"));
    }

    fs.Write(","u8);
    var sus = (from su in attr select (su.Time, su.Code)).ToImmutableSortedSet();
    foreach (var (i, (time, suCode)) in sus.Indexed())
    {
        if (i != 0)
        {
            fs.Write(";"u8);
        }
        fs.Write(Encoding.UTF8.GetBytes($"{suCode}"));
        if (end != time)
        {
            fs.Write(Encoding.UTF8.GetBytes($"[{time}]"));
        }
    }
    fs.Write("\n"u8);
}
