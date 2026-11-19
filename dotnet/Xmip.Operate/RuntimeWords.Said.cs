using System.Text.Json;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

// Section 28.3 as a sentence is said: each asked of the runtime when it is
// said, in its words, a longer answer asked again at its length.
public sealed partial class RuntimeWords
{
    /// <summary>The sentence <paramref name="which"/>
    /// (<see cref="OperateAbi.Sentence"/>) says of <paramref name="scope"/> on
    /// the surface <paramref name="source"/>.</summary>
    public string Sentence(int which, string scope, string source)
    {
        return Two(_sentence, which, scope, source);
    }

    /// <summary>An act's <paramref name="outcome"/> and the node's
    /// <paramref name="result"/>, as a person reads them.</summary>
    public string ActSaid(int outcome, string result)
    {
        return One(_actSaid, outcome, result);
    }

    /// <summary>How long before <paramref name="now"/> <paramref name="observed"/>
    /// was.</summary>
    public string Age(DateTimeOffset observed, DateTimeOffset now)
    {
        long from = Nanos(observed);
        long to = Nanos(now);
        unsafe
        {
            return Said((into, cap, length) => _age(from, to, into, cap, length));
        }
    }

    /// <summary>A figure with its thousands grouped, or a dash for none.</summary>
    public string Figure(ulong? value)
    {
        byte has = value.HasValue ? (byte)1 : (byte)0;
        ulong figure = value ?? 0;
        unsafe
        {
            return Said((into, cap, length) => _figure(has, figure, into, cap, length));
        }
    }

    /// <summary>The six figures, in their order, in one line.</summary>
    public string FiguresLine(IReadOnlyList<ulong?> six)
    {
        ArgumentNullException.ThrowIfNull(six);
        byte[] has = new byte[6];
        ulong[] values = new ulong[6];
        for (int at = 0; at < 6; at++)
        {
            has[at] = six[at].HasValue ? (byte)1 : (byte)0;
            values[at] = six[at] ?? 0;
        }

        unsafe
        {
            fixed (byte* given = has)
            fixed (ulong* figures = values)
            {
                byte* first = given;
                ulong* second = figures;
                return Said((into, cap, length) =>
                    _figuresLine(first, second, into, cap, length));
            }
        }
    }

    /// <summary><paramref name="value"/> as a surface shows it
    /// (<see cref="OperateAbi.Text"/>).</summary>
    public string Text(int which, string value)
    {
        return One(_text, which, value);
    }

    /// <summary>Who acts: <paramref name="stated"/> where it names someone,
    /// else <paramref name="user"/>.</summary>
    public string Who(string? stated, string user)
    {
        return Answered(_who, stated ?? string.Empty, user);
    }

    /// <summary>The sentence the verdict on <paramref name="path"/> says.</summary>
    public string Validated(string path, int status, IReadOnlyList<string> problems)
    {
        ArgumentNullException.ThrowIfNull(problems);
        Utf8Pack pack = new([path, .. problems]);
        XmipStr[] listed = new XmipStr[Math.Max(problems.Count, 1)];
        unsafe
        {
            fixed (byte* data = pack.Bytes)
            fixed (XmipStr* each = listed)
            {
                XmipStr named = pack.Borrow(data, 0);
                for (int at = 0; at < problems.Count; at++)
                {
                    each[at] = pack.Borrow(data, at + 1);
                }

                XmipStr* problem = each;
                nuint count = (nuint)problems.Count;
                return Said((into, cap, length) =>
                    _validated(named, status, problem, count, into, cap, length));
            }
        }
    }

    /// <summary>The run <paramref name="run"/>, as <c>observe::Run</c>'s JSON,
    /// said.</summary>
    public string RunSaid(string run)
    {
        return Answered(_runSaid, run, string.Empty);
    }

    /// <summary>A node's declaration, as its JSON, said.</summary>
    public string CapabilitySaid(string declaration)
    {
        return Answered(_capabilitySaid, declaration, string.Empty);
    }

    /// <summary>The places in <paramref name="entries"/> — the list
    /// <paramref name="list"/> as its section's JSON — that
    /// <paramref name="query"/> chooses, in its order; throws
    /// <see cref="ArgumentException"/> with the REFUSED sentence.</summary>
    public JsonDocument ListQuery(int list, string entries, string query)
    {
        Utf8Pack pack = new([entries, query]);
        int status = 0;
        string answer;
        unsafe
        {
            fixed (byte* data = pack.Bytes)
            {
                XmipStr given = pack.Borrow(data, 0);
                XmipStr asked = pack.Borrow(data, 1);
                answer = Said(
                    (into, cap, length) =>
                        status = _listQuery(list, given, asked, into, cap, length),
                    check: false);
            }
        }

        return status == (int)XmipStatus.Ok
            ? JsonDocument.Parse(answer)
            : throw new ArgumentException(answer);
    }

    /// <summary>The word <paramref name="which"/>
    /// (<see cref="OperateAbi.ListWord"/>) a row shows, from its two texts.</summary>
    public string ListWord(int which, string first, string second)
    {
        return Two(_listWords, which, first, second);
    }
}
