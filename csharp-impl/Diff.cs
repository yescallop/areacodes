namespace Areacodes;

record FwdDiff(uint Time, uint Code, bool Internal, bool Optional, List<uint> Attr);

abstract record Selector
{
    public record GivenName(string Name, string? Parent) : Selector
    {
        public static GivenName Parse(string curName, string selStr)
        {
            string? parent = null;
            var parts = selStr.SplitOnce('(');
            if (parts.HasValue)
            {
                var (name, rest) = parts.Value;
                if (!rest.EndsWith(")"))
                {
                    throw new FormatException();
                }
                parent = rest[..(rest.Length - 1)];
                selStr = name;
            }

            if (selStr == "#")
            {
                selStr = curName;
            }
            return new GivenName(selStr, parent);
        }
    }
    public record CurCode : Selector;
    public record ParentCode : Selector;
}

class DataTable
{
    Dictionary<uint, string> _codeToName;
    Dictionary<string, List<uint>> _nameToCode;
    HashSet<uint> _hasChildren;

    public DataTable()
    {
        _codeToName = new(4096);
        _nameToCode = new(4096);
        _hasChildren = new(512);
    }

    public string? NameByCode(uint code)
    {
        if (!_codeToName.TryGetValue(code, out string? name) && code == 0)
        {
            name = "中华人民共和国";
        }
        return name;
    }

    public uint ParentCode(uint code)
    {
        code = Utils.ParentCode(code);
        return _codeToName.ContainsKey(code) ? code : Utils.ParentCode(code);
    }

    public List<uint>? CodesByName(string name)
    {
        _nameToCode.TryGetValue(name, out var codes);
        return codes;
    }

    public bool HasChildren(uint code)
    {
        return _hasChildren.Contains(code);
    }

    public void Insert(uint code, string name)
    {
        _codeToName.Add(code, name);

        if (!_nameToCode.TryGetValue(name, out var list))
        {
            list = _nameToCode[name] = new List<uint>();
        }
        list.Add(code);

        _hasChildren.Add(ParentCode(code));
    }

    public void Clear()
    {
        _codeToName.Clear();
        _nameToCode.Clear();
        _hasChildren.Clear();
    }
}

record Line(bool Fwd, bool Internal, uint Code, string Name, List<Selector> Attr)
{
    public static Line? Parse(string line)
    {
        if (line.Length == 0)
        {
            return null;
        }

        bool fwd = false;
        bool _internal = false;
        switch (line[0])
        {
            case '-':
                fwd = true;
                break;
            case '+':
                break;
            case '=':
                _internal = true;
                break;
            case '#':
                return null;
            default:
                throw new FormatException();
        }

        if (line.Length < 8)
        {
            throw new FormatException();
        }

        var code = uint.Parse(line[1..7]);
        if (line[7] != '\t')
        {
            throw new FormatException();
        }

        line = line[8..];
        var (name, attrStr) = line.SplitOnce('>', '<') ?? throw new FormatException();

        bool actualFwd = line[name.Length] == '>';
        if (_internal)
        {
            fwd = actualFwd;
        }
        else if (actualFwd != fwd)
        {
            throw new FormatException();
        }

        var attr = new List<Selector>();
        foreach (string selStr in attrStr.Split(','))
        {
            if (selStr.EndsWith('?'))
            {
                continue;
            }
            string selStrStripped = selStr.EndsWith('!')
                ? selStr[..(selStr.Length - 1)] : selStr;
            Selector sel = selStrStripped switch
            {
                "." => new Selector.CurCode(),
                ".." => new Selector.ParentCode(),
                _ => Selector.GivenName.Parse(name, selStrStripped),
            };
            attr.Add(sel);
        }

        return new Line(fwd, _internal, code, name, attr);
    }
}

static class Diff
{
    public static IEnumerable<FwdDiff> ReadFwdDiff()
    {
        var src = new DataTable();
        var dst = new DataTable();
        var rem = new Dictionary<int, HashSet<int>>(1024);
        var attr = new List<uint>();

        foreach (string path in Directory.GetFiles(Constants.DiffDirectory))
        {
            string fileStem = Path.GetFileNameWithoutExtension(path);
            var (srcYear, dstYear) = fileStem.SplitOnce('-') ??
                throw new InvalidDataException($"{fileStem}: invalid file stem");

            Console.WriteLine($"----- {fileStem} -----");

            foreach (var (code, name) in Utils.ReadData($"{Constants.DataDirectory}/{srcYear}.txt"))
            {
                src.Insert(code, name);
            }
            foreach (var (code, name) in Utils.ReadData($"{Constants.DataDirectory}/{dstYear}.txt"))
            {
                dst.Insert(code, name);
            }

            var time = uint.Parse(dstYear);

            foreach (var (i, lineStr) in File.ReadLines(path).Indexed())
            {
                Line? line;
                try
                {
                    line = Line.Parse(lineStr);
                }
                catch (FormatException)
                {
                    throw new InvalidDataException($"line {i}: invalid format");
                }
                if (line == null)
                {
                    continue;
                }

                uint code = line.Code;
                string name = line.Name;

                if (line.Internal)
                {
                    string? srcName = src.NameByCode(code);
                    string? dstName = dst.NameByCode(code);
                    if (srcName != dstName || srcName != name)
                    {
                        throw new InvalidDataException($"{code}: invalid internal change");
                    }
                }
                else if (line.Fwd)
                {
                    if (src.NameByCode(code) != name)
                    {
                        throw new InvalidDataException($"{code}: invalid deletion");
                    }
                    if (dst.NameByCode(code) == name)
                    {
                        Console.WriteLine($"{code}: same-name deletion");
                    }
                }
                else
                {
                    if (dst.NameByCode(code) != name)
                    {
                        throw new InvalidDataException($"{code}: invalid addition");
                    }
                    if (src.NameByCode(code) == name)
                    {
                        Console.WriteLine($"{code}: same-name addition");
                    }
                }

                var (table, origin) = line.Fwd ? (dst, src) : (src, dst);

                attr.Clear();
                Select(table, origin, rem, line, attr);

                if (attr.Count == 0)
                {
                    throw new InvalidDataException($"{code}: empty attr");
                }
                bool optional = origin.HasChildren(code);

                if (line.Fwd)
                {
                    yield return new FwdDiff(time, code, line.Internal, optional, attr);
                }
                else
                {
                    foreach (uint selCode in attr)
                    {
                        yield return new FwdDiff(time, selCode, line.Internal, optional, new List<uint>() { code });
                    }
                }
            }

            foreach (var (code, remCodes) in rem)
            {
                foreach (int remCode in remCodes)
                {
                    if (rem.ContainsKey(remCode))
                    {
                        Console.WriteLine($"{remCode}@{code}: asymmetry found");
                    }
                }
            }

            src.Clear();
            dst.Clear();
            rem.Clear();
        }
    }

    static void Select(DataTable table, DataTable origin, Dictionary<int, HashSet<int>> rem, Line line, List<uint> res)
    {
        uint code = line.Code;

        foreach (Selector sel in line.Attr)
        {
            uint resCode;
            switch (sel)
            {
                case Selector.GivenName:
                    (string name, string? parent) = (Selector.GivenName)sel;
                    List<uint> selCodes = table.CodesByName(name) ??
                        throw new InvalidDataException($"{name}@{code}: not found");

                    uint minDist = 4;
                    uint cnt = 0;
                    resCode = 0;

                    foreach (uint selCode in selCodes)
                    {
                        if (parent != null
                            && table.NameByCode(table.ParentCode(selCode)) != parent)
                        {
                            continue;
                        }

                        uint dist = Utils.CodeDistance(code, selCode);
                        if (dist == 1
                            && table.NameByCode(table.ParentCode(selCode))
                                != origin.NameByCode(origin.ParentCode(selCode)))
                        {
                            dist = 2;
                        }

                        if (dist < minDist)
                        {
                            minDist = dist;
                            cnt = 1;
                            resCode = selCode;
                        }
                        else if (dist == minDist)
                        {
                            cnt += 1;
                        }
                    }

                    if (cnt == 0)
                    {
                        throw new InvalidDataException($"{name}@{code}: not found");
                    }
                    else if (cnt != 1)
                    {
                        throw new InvalidDataException($"{name}@{code}: multiple records found");
                    }
                    break;
                case Selector.CurCode:
                    resCode = code;
                    if (table.NameByCode(code) == null)
                    {
                        throw new InvalidDataException($"{code}: not found");
                    }
                    break;
                case Selector.ParentCode:
                    resCode = origin.ParentCode(code);
                    string? parentName = table.NameByCode(resCode);
                    if (parentName != null)
                    {
                        // Console.WriteLine($"..@{code} {line.Name} = {parentName}");
                    }
                    else
                    {
                        throw new InvalidDataException($"..@{code}: not found");
                    }
                    break;
                default:
                    throw new InvalidOperationException("unreachable");
            }

            res.Add(resCode);

            // Asymmetry check
            var (i_code, i_resCode) = (-(int)code, (int)resCode);
            if (!line.Fwd)
            {
                (i_code, i_resCode) = (-i_code, -i_resCode);
            }

            bool insert = true;
            if (rem.TryGetValue(i_resCode, out HashSet<int>? remSet))
            {
                if (remSet.Remove(i_code))
                {
                    insert = false;
                }
            }

            if (!rem.TryGetValue(i_code, out remSet))
            {
                remSet = rem[i_code] = new HashSet<int>();
            }
            if (insert)
            {
                remSet.Add(i_resCode);
            }
        }
    }
}

