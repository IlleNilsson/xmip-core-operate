using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>Section 28.1 of <c>include/xmip_operate.h</c>: a publication's
/// scope tree, built once and asked by scope, every record answered by its
/// place in what it was built from — <c>operate::surface::scope_index</c>,
/// the one every surface reads (ADR-0052, amendment 2026-11-18).</summary>
[SkipLocalsInit]
public sealed partial class RuntimeScopeIndexes
{
    // A scope is short; up to this many bytes it is encoded on the stack.
    private const int Small = 512;

    // Children of one scope asked for at once, before asking again at the
    // count the runtime said.
    private const int Room = 32;

    private readonly delegate* unmanaged[Cdecl]<
        XmipHealthEntry*, nuint, XmipMeasurement*, nuint, ulong, XmipStr, nint*, int> _build;
    private readonly delegate* unmanaged[Cdecl]<nint, ulong, XmipStr, nint*, int> _ofPublication;
    private readonly delegate* unmanaged[Cdecl]<nint, void> _free;
    private readonly delegate* unmanaged[Cdecl]<nint, XmipScopeIndexFacts*, int> _facts;
    private readonly delegate* unmanaged[Cdecl]<nint, XmipStr, XmipScopePlaced*, int> _placed;
    private readonly delegate* unmanaged[Cdecl]<
        nint, XmipStr, XmipScopePlaced*, nuint, nuint*, int> _children;

    internal RuntimeScopeIndexes(nint library)
    {
        _build = (delegate* unmanaged[Cdecl]<
            XmipHealthEntry*, nuint, XmipMeasurement*, nuint, ulong, XmipStr, nint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexBuildEntrypoint);
        _ofPublication = (delegate* unmanaged[Cdecl]<nint, ulong, XmipStr, nint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexOfPublicationEntrypoint);
        _free = (delegate* unmanaged[Cdecl]<nint, void>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexFreeEntrypoint);
        _facts = (delegate* unmanaged[Cdecl]<nint, XmipScopeIndexFacts*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexFactsEntrypoint);
        _placed = (delegate* unmanaged[Cdecl]<nint, XmipStr, XmipScopePlaced*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexPlacedEntrypoint);
        _children = (delegate* unmanaged[Cdecl]<
            nint, XmipStr, XmipScopePlaced*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexChildrenEntrypoint);
        _list = (delegate* unmanaged[Cdecl]<nint, uint, XmipStr, XmipStr*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexListEntrypoint);
        _label = (delegate* unmanaged[Cdecl]<nint, XmipStr, XmipStr*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexLabelEntrypoint);
        _worstAtStage = (delegate* unmanaged[Cdecl]<nint, XmipStr, long*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexWorstAtStageEntrypoint);
        _measure = (delegate* unmanaged[Cdecl]<nint, XmipStr, int, long, XmipMeasurement*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexMeasureEntrypoint);
        _capabilities = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexCapabilitiesEntrypoint);
        _select = (delegate* unmanaged[Cdecl]<nint, XmipStr, XmipStr, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeIndexSelectEntrypoint);
        _relative = (delegate* unmanaged[Cdecl]<XmipStr, uint, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ScopeRelativeEntrypoint);
        _sum = (delegate* unmanaged[Cdecl]<XmipFigures*, nuint, XmipFigures*, int>)
            NativeLibrary.GetExport(library, OperateAbi.FiguresSumEntrypoint);
    }

    /// <summary>The symbols it binds, each of which a runtime exports, its
    /// remote build too.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } =
    [
        OperateAbi.ScopeIndexBuildEntrypoint,
        OperateAbi.ScopeIndexOfPublicationEntrypoint,
        OperateAbi.ScopeIndexFreeEntrypoint,
        OperateAbi.ScopeIndexFactsEntrypoint,
        OperateAbi.ScopeIndexPlacedEntrypoint,
        OperateAbi.ScopeIndexChildrenEntrypoint,
        OperateAbi.ScopeIndexListEntrypoint,
        OperateAbi.ScopeIndexLabelEntrypoint,
        OperateAbi.ScopeIndexWorstAtStageEntrypoint,
        OperateAbi.ScopeIndexMeasureEntrypoint,
        OperateAbi.ScopeIndexCapabilitiesEntrypoint,
        OperateAbi.ScopeIndexSelectEntrypoint,
        OperateAbi.ScopeRelativeEntrypoint,
        OperateAbi.FiguresSumEntrypoint,
    ];

    /// <summary>
    /// The tree <paramref name="records"/> and <paramref name="counts"/>
    /// make, each record keeping its place; said to be
    /// <paramref name="revision"/> of <paramref name="source"/>.
    /// </summary>
    public ScopeIndexHandle Build(
        IReadOnlyList<HealthRecord> records,
        IReadOnlyList<(string Scope, Counted Counted, ulong Value, DateTimeOffset? Observed)>
            counts,
        ulong revision,
        string source)
    {
        ArgumentNullException.ThrowIfNull(records);
        ArgumentNullException.ThrowIfNull(counts);
        ArgumentNullException.ThrowIfNull(source);

        // Every text encoded once, end to end in one buffer: each record's
        // scope and evidence, then each count's scope, then the source.
        List<string> texts = [with((records.Count * 2) + counts.Count + 1)];
        texts.AddRange(records.SelectMany(record => new[] { record.Scope, record.Evidence }));
        texts.AddRange(counts.Select(count => count.Scope));
        texts.Add(source);
        Utf8Pack pack = new(texts);
        XmipHealthEntry[] entries = new XmipHealthEntry[records.Count];
        XmipMeasurement[] measured = new XmipMeasurement[counts.Count];
        nint index = 0;
        int status;

        unsafe
        {
            fixed (byte* data = pack.Bytes)
            fixed (XmipHealthEntry* into = entries)
            fixed (XmipMeasurement* counted = measured)
            {
                for (int at = 0; at < entries.Length; at++)
                {
                    HealthRecord record = records[at];
                    into[at].Scope = pack.Borrow(data, at * 2);
                    into[at].Health = (int)record.State;
                    into[at].Severity = record.Severity;
                    into[at].Evidence = pack.Borrow(data, (at * 2) + 1);
                    into[at].ObservedUnixNanos = UnixNanos(record.Observed);
                }

                for (int at = 0; at < measured.Length; at++)
                {
                    long observed = counts[at].Observed is { } seen ? UnixNanos(seen) : 0;
                    counted[at].Scope = pack.Borrow(data, (records.Count * 2) + at);
                    counted[at].Counted = (int)counts[at].Counted;
                    counted[at].Value = counts[at].Value;
                    counted[at].WindowStartUnixNanos = observed;
                    counted[at].WindowEndUnixNanos = observed;
                    counted[at].ObservedUnixNanos = observed;
                }

                status = _build(
                    into, (nuint)entries.Length, counted, (nuint)measured.Length, revision,
                    pack.Borrow(data, pack.Count - 1), &index);
            }
        }

        Check(status);
        return new ScopeIndexHandle(index, _free);
    }
    /// <summary>The tree of a read publication's records and counts, each
    /// record at the place <see cref="PublicationReader"/> hands it out; the
    /// publication may be freed after.</summary>
    public ScopeIndexHandle OfPublication(nint publication, ulong revision, string source)
    {
        ArgumentNullException.ThrowIfNull(source);

        using PinnedStr said = XmipStr.Pin(source);
        nint index = 0;
        unsafe
        {
            Check(_ofPublication(publication, revision, said.Value, &index));
        }
        return new ScopeIndexHandle(index, _free);
    }

    /// <summary>An index the runtime handed over, held from now on (section
    /// 28.2's surfaces hand theirs).</summary>
    public ScopeIndexHandle Adopt(nint index)
    {
        return new ScopeIndexHandle(index, _free);
    }

    /// <summary>What <paramref name="index"/> is: its revision, how many
    /// records and scopes, when its newest figure was observed and its
    /// source.</summary>
    public (ulong Revision, int Leaves, int Scopes, DateTimeOffset? Observed, string Source)
        Facts(ScopeIndexHandle index)
    {
        ArgumentNullException.ThrowIfNull(index);

        XmipScopeIndexFacts facts;
        string source;

        unsafe
        {
            Check(_facts(index.DangerousGetHandle(), &facts));
            source = facts.Source.Read();
        }

        GC.KeepAlive(index);

        return (
            facts.Revision,
            checked((int)facts.Leaves),
            checked((int)facts.Scopes),
            facts.Observed != 0 ? Operator.FromNanos(facts.ObservedUnixNanos) : null,
            source);
    }

    /// <summary><paramref name="scope"/> as <paramref name="index"/> holds
    /// it, or null where it holds no such scope.</summary>
    public ScopePlacement? Placed(ScopeIndexHandle index, string scope)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(scope);

        unsafe
        {
            XmipScopePlaced placed;
            Span<byte> room = stackalloc byte[Small];
            byte[] text = Encoded(scope, room, out int length);
            int status;

            fixed (byte* data = text.Length > 0 ? text : room)
            {
                status = _placed(
                    index.DangerousGetHandle(), new XmipStr(data, (nuint)length), &placed);
            }

            if (status == (int)XmipStatus.NotFound)
            {
                GC.KeepAlive(index);
                return null;
            }

            Check(status);
            ScopePlacement copied = Copied(placed);
            GC.KeepAlive(index);
            return copied;
        }
    }

    /// <summary>The scopes directly beneath <paramref name="scope"/>, worst
    /// first; none beneath a scope the tree lacks.</summary>
    public IReadOnlyList<ScopePlacement> Children(ScopeIndexHandle index, string scope)
    {
        ArgumentNullException.ThrowIfNull(index);
        ArgumentNullException.ThrowIfNull(scope);

        unsafe
        {
            XmipScopePlaced[] room = new XmipScopePlaced[Room];
            Span<byte> stack = stackalloc byte[Small];
            byte[] text = Encoded(scope, stack, out int length);
            nuint count = 0;

            fixed (byte* data = text.Length > 0 ? text : stack)
            {
                XmipStr asked = new(data, (nuint)length);

                for (int round = 0; round < 2; round++)
                {
                    fixed (XmipScopePlaced* into = room)
                    {
                        Check(_children(
                            index.DangerousGetHandle(), asked, into, (nuint)room.Length,
                            &count));
                    }

                    if (count <= (nuint)room.Length)
                    {
                        break;
                    }

                    room = new XmipScopePlaced[checked((int)count)];
                }
            }

            ScopePlacement[] children = [.. room.Take((int)count).Select(Copied)];
            GC.KeepAlive(index);
            return children;
        }
    }

    // The one copy of a placed scope out of the index.
    private static ScopePlacement Copied(XmipScopePlaced placed)
    {
        int[] leaves = new int[(int)placed.LeavesLen];
        string scope;
        string label;
        string stage;

        unsafe
        {
            if (leaves.Length > 0)
            {
                Marshal.Copy((nint)placed.Leaves, leaves, 0, leaves.Length);
            }

            scope = placed.Scope.Read();
            label = placed.Label.Read();
            stage = placed.Stage.Read();
        }

        ulong[] values =
        [
            placed.Streams, placed.Messages, placed.Journeys,
            placed.Bytes, placed.Retrying, placed.Failed,
        ];

        return new ScopePlacement(
            scope,
            label,
            leaves,
            placed.Own,
            placed.Mood < 0 ? null : (HealthState)placed.Mood,
            placed.RowMood < 0 ? null : (HealthState)placed.RowMood,
            placed.IsLeaf != 0,
            placed.IsContainer != 0,
            placed.BeginsStage != 0,
            stage,
            [.. values.Select((value, kind) => (placed.Published & (1 << kind)) != 0
                ? (ulong?)value
                : null)]);
    }

    // `text` encoded into `room` where it fits, else into an array of its
    // own, which is then answered; empty where `room` holds it.
    internal static byte[] Encoded(string text, Span<byte> room, out int length)
    {
        if (Encoding.UTF8.GetMaxByteCount(text.Length) <= room.Length)
        {
            length = Encoding.UTF8.GetBytes(text, room);
            return [];
        }

        byte[] own = Encoding.UTF8.GetBytes(text);
        length = own.Length;
        return own;
    }

    /// <summary>A time as the header carries one, nanoseconds since the Unix
    /// epoch, held at the most a 64-bit count says for a time beyond it.</summary>
    public static long UnixNanos(DateTimeOffset at)
    {
        long ticks = (at - DateTimeOffset.UnixEpoch).Ticks;

        return ticks > long.MaxValue / 100 ? long.MaxValue
            : ticks < long.MinValue / 100 ? long.MinValue
            : ticks * 100;
    }

    private static void Check(int status)
    {
        if (status != 0)
        {
            throw new InvalidOperationException(
                $"the runtime answered a scope tree with {(XmipStatus)status}");
        }
    }
}
