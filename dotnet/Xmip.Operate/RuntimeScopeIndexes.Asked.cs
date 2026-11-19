using System.Text;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

// What a scope tree answers besides a scope — a list, a label, the worst on
// a stage, one figure, every declaration, what a pattern selects — and the
// tree's composed rules: a scope's relatives and figures summed.
public sealed partial class RuntimeScopeIndexes
{
    // The first room a text answer is asked with.
    private const int TextRoom = 256;

    private readonly delegate* unmanaged[Cdecl]<
        nint, uint, XmipStr, XmipStr*, nuint, nuint*, int> _list;
    private readonly delegate* unmanaged[Cdecl]<nint, XmipStr, XmipStr*, int> _label;
    private readonly delegate* unmanaged[Cdecl]<nint, XmipStr, long*, int> _worstAtStage;
    private readonly delegate* unmanaged[Cdecl]<
        nint, XmipStr, int, long, XmipMeasurement*, int> _measure;
    private readonly delegate* unmanaged[Cdecl]<nint, byte*, nuint, nuint*, int> _capabilities;
    private readonly delegate* unmanaged[Cdecl]<
        nint, XmipStr, XmipStr, byte*, nuint, nuint*, int> _select;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, uint, byte*, nuint, nuint*, int> _relative;
    private readonly delegate* unmanaged[Cdecl]<XmipFigures*, nuint, XmipFigures*, int> _sum;

    /// <summary>One of the index's lists, in ordinal order: where the stage
    /// <paramref name="argument"/> begins
    /// (<see cref="OperateAbi.ScopeListStageScopes"/>), the names beneath the
    /// root (<see cref="OperateAbi.ScopeListNodes"/>), or what the pattern
    /// <paramref name="argument"/> matches
    /// (<see cref="OperateAbi.ScopeListMatching"/>).</summary>
    public IReadOnlyList<string> List(ScopeIndexHandle index, uint which, string argument)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(argument);

        using PinnedStr asked = XmipStr.Pin(argument);
        XmipStr[] room = new XmipStr[Room];
        nuint count = 0;

        for (int round = 0; round < 2; round++)
        {
            fixed (XmipStr* into = room)
            {
                unsafe
                {
                    Check(_list(
                        index.DangerousGetHandle(), which, asked.Value, into,
                        (nuint)room.Length, &count));
                }
            }

            if (count <= (nuint)room.Length)
            {
                break;
            }

            room = new XmipStr[checked((int)count)];
        }

        string[] listed = new string[(int)count];

        unsafe
        {
            for (int at = 0; at < listed.Length; at++)
            {
                listed[at] = room[at].Read();
            }
        }

        GC.KeepAlive(index);
        return listed;
    }

    /// <summary>What a person reads <paramref name="scope"/> as.</summary>
    public string Label(ScopeIndexHandle index, string scope)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(scope);

        using PinnedStr asked = XmipStr.Pin(scope);
        XmipStr label;
        string read;

        unsafe
        {
            Check(_label(index.DangerousGetHandle(), asked.Value, &label));
            read = label.Read();
        }

        GC.KeepAlive(index);
        return read;
    }

    /// <summary>The place of the worst record on the stage
    /// <paramref name="stage"/>, -1 for none.</summary>
    public long WorstAtStage(ScopeIndexHandle index, string stage)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(stage);

        using PinnedStr asked = XmipStr.Pin(stage);
        long place;

        unsafe
        {
            Check(_worstAtStage(index.DangerousGetHandle(), asked.Value, &place));
        }

        GC.KeepAlive(index);
        return place;
    }

    /// <summary>One figure at <paramref name="scope"/> as a measurement, or
    /// null where nothing of that kind is counted there.</summary>
    public MeasurementRecord? Measure(
        ScopeIndexHandle index, string scope, Counted counted, DateTimeOffset now)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(scope);

        using PinnedStr asked = XmipStr.Pin(scope);
        XmipMeasurement measured;
        int status;

        unsafe
        {
            status = _measure(
                index.DangerousGetHandle(), asked.Value, (int)counted, UnixNanos(now), &measured);
        }

        GC.KeepAlive(index);

        if (status == (int)XmipStatus.NotFound)
        {
            return null;
        }

        Check(status);
        return new MeasurementRecord(
            scope,
            counted,
            measured.Value,
            Operator.FromNanos(measured.WindowStartUnixNanos),
            Operator.FromNanos(measured.WindowEndUnixNanos),
            Operator.FromNanos(measured.ObservedUnixNanos));
    }

    /// <summary>What each node declared by its last capability record, as the
    /// header's JSON array.</summary>
    public string Capabilities(ScopeIndexHandle index)
    {
        ArgumentNullException.ThrowIfNull(index);

        byte[] room = new byte[TextRoom];
        nuint needed = 0;

        for (int round = 0; round < 2; round++)
        {
            fixed (byte* into = room)
            {
                unsafe
                {
                    Check(_capabilities(
                        index.DangerousGetHandle(), into, (nuint)room.Length, &needed));
                }
            }

            if (needed <= (nuint)room.Length)
            {
                break;
            }

            room = new byte[checked((int)needed)];
        }

        GC.KeepAlive(index);
        return Encoding.UTF8.GetString(room, 0, (int)needed);
    }

    /// <summary>What <paramref name="pattern"/> selects among the index's
    /// scopes, as the header's JSON; false with the refusal, naming
    /// <paramref name="source"/>, where a wildcard matches nothing.</summary>
    public bool Select(ScopeIndexHandle index, string pattern, string source, out string answer)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(pattern);
        ArgumentNullException.ThrowIfNull(source);

        using PinnedStr asked = XmipStr.Pin(pattern);
        using PinnedStr named = XmipStr.Pin(source);
        byte[] room = new byte[TextRoom];
        nuint needed = 0;
        int status = 0;

        for (int round = 0; round < 2; round++)
        {
            fixed (byte* into = room)
            {
                unsafe
                {
                    status = _select(
                        index.DangerousGetHandle(), asked.Value, named.Value, into,
                        (nuint)room.Length, &needed);
                }
            }

            if (needed <= (nuint)room.Length)
            {
                break;
            }

            room = new byte[checked((int)needed)];
        }

        GC.KeepAlive(index);
        answer = Encoding.UTF8.GetString(room, 0, (int)needed);

        if (status == (int)XmipStatus.NotFound)
        {
            return false;
        }

        Check(status);
        return true;
    }

    /// <summary><paramref name="scope"/>'s parent
    /// (<see cref="OperateAbi.ScopeRelativeParent"/>), its node's own scope
    /// (<see cref="OperateAbi.ScopeRelativeNodeScope"/>) or its name
    /// (<see cref="OperateAbi.ScopeRelativeName"/>).</summary>
    public string Relative(string scope, uint which)
    {
        ArgumentNullException.ThrowIfNull(scope);

        unsafe
        {
            Span<byte> stack = stackalloc byte[Small];
            byte[] text = Encoded(scope, stack, out int length);
            Span<byte> answer = stackalloc byte[Small];
            nuint needed = 0;

            fixed (byte* data = text.Length > 0 ? text : stack)
            {
                XmipStr asked = new(data, (nuint)length);

                fixed (byte* into = answer)
                {
                    Check(_relative(asked, which, into, (nuint)answer.Length, &needed));
                }

                if (needed > (nuint)answer.Length)
                {
                    byte[] more = new byte[checked((int)needed)];

                    fixed (byte* into = more)
                    {
                        Check(_relative(asked, which, into, (nuint)more.Length, &needed));
                    }

                    return Encoding.UTF8.GetString(more, 0, (int)needed);
                }
            }

            return Encoding.UTF8.GetString(answer[..(int)needed]);
        }
    }

    /// <summary><paramref name="parts"/> summed: a kind no part published
    /// stays unpublished, the newest observed wins.</summary>
    public XmipFigures Sum(ReadOnlySpan<XmipFigures> parts)
    {
        XmipFigures sum;

        fixed (XmipFigures* from = parts)
        {
            unsafe
            {
                Check(_sum(from, (nuint)parts.Length, &sum));
            }
        }

        return sum;
    }
}
